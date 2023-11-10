use crate::api::schema::*;
use crate::log::logging::*;
use crate::manifests::catalogs::*;
use semver::{BuildMetadata, Prerelease, Version};
use std::cmp::*;
use std::fs;

// list all components in the current operator image index
pub async fn list_components(log: &Logging, dir: String, filter: FilterConfig) {
    if filter.operators.is_some() {
        for operator in filter.operators.unwrap() {
            let dc = read_operator_catalog(dir.to_string() + &"/".to_string() + &operator.name);
            list_channel_info(log, dc.unwrap(), operator);
        }
    } else {
        // no entry for operators, so traverse through all operators
        // in the given catalog
        let paths = fs::read_dir(&dir).unwrap();
        for path in paths {
            let entry = path.expect("could not resolve path entry");
            let dir_name = entry.path();
            let str_dir = dir_name.into_os_string().into_string().unwrap();
            let res = str_dir.split("/");
            let n = format!("{}", res.into_iter().last().unwrap());
            let dc = read_operator_catalog(str_dir);
            let operator = Operator {
                name: n,
                channel: Some("all".to_string()),
                from_version: Some("0.0.0".to_string()),
            };
            list_channel_info(log, dc.unwrap(), operator);
        }
    }
}

// iterate through object and display values
pub fn list_channel_info(log: &Logging, input: serde_json::Value, filter: Operator) {
    // parse the Package and Channel parts of the catalog.json
    let pkg: Vec<Package> = match serde_json::from_value(input.clone()) {
        Ok(val) => val,
        Err(error) => panic!("error {}", error),
    };

    let ch: Vec<Channel> = match serde_json::from_value(input) {
        Ok(val) => val,
        Err(error) => panic!("error {}", error),
    };

    // check to see if filter.from_version is valid (or empty)
    let mut current_semver = Version::parse("0.0.0").unwrap();
    let mut current_version = String::from("0.0.0");
    log.trace(&format!("current version {}", current_version));

    if filter.from_version.is_some() {
        current_version = filter.from_version.unwrap();
        current_semver = Version::parse(&current_version).unwrap();
    }

    // check to see if filter.channel is valid (or empty)
    let mut current_channel = String::from("all");
    if filter.channel.is_some() {
        current_channel = filter.channel.unwrap();
    }

    let package = pkg.into_iter().nth(0).unwrap();
    log.hi(&format!("operator '{}'", package.name,));
    log.ex(&format!(
        "  defaultChannel {:?}",
        package.default_channel.unwrap()
    ));

    let mut available_versions: Vec<String> = vec![];
    // iterate through the dc - look specifically for olm.channel schema
    for x in ch {
        if x.schema == "olm.channel" {
            if current_channel == x.name || current_channel == "all" {
                let mut current: Vec<ChannelEntry> = vec![];
                let mut replace: Vec<String> = vec![];
                //let mut skip_range: Vec<String> = vec![];
                //let mut skip_tracker: Vec<String> = vec![];

                // entries contain all the relevant upgrade path info
                for y in x.entries.unwrap() {
                    // get the bundle semver
                    if !available_versions.contains(&y.name.clone()) {
                        available_versions.insert(0, y.name.clone());
                    }
                    let mut semver_tmp = String::from("0.0.0");
                    if semver_tmp != "0.0.0" {}
                    if y.name.clone().contains(".v") {
                        semver_tmp = y.name.split(".v").nth(1).unwrap().to_string();
                    } else {
                        // the case when we don't have ".v" in the catalog
                        // oh the joys of giving devs free range :(
                        // for now we only do major,min,patch,pre and ignore build versions
                        let n = y.name.split(".").nth(0).unwrap().to_string();
                        semver_tmp = y
                            .name
                            .split(&n)
                            .nth(1)
                            .unwrap()
                            .to_string()
                            .get(1..)
                            .unwrap()
                            .to_string();
                    }
                    let bundle_semver = build_semver(semver_tmp);
                    let res = current_semver.cmp(&bundle_semver);
                    if res != Ordering::Greater {
                        current.insert(0, y.clone());
                        if y.replaces.is_some() {
                            replace.insert(0, y.replaces.unwrap());
                        }
                    }
                }
                let mut stage: Vec<ChannelEntry> = vec![];
                let mut updated: Vec<ChannelEntry> = vec![];
                log.mid(&format!("  channel name {}", x.name));

                // TODO: this can be re-factored into a couple of lines
                for ce in current.iter() {
                    let mut found = false;
                    for r in replace.iter() {
                        if r == &ce.name {
                            found = true;
                        }
                    }
                    if !found {
                        stage.insert(0, ce.clone());
                    }
                }
                for n in stage.iter() {
                    let mut found = false;
                    for ce in current.iter() {
                        if ce.skips.is_some() {
                            for s in ce.skips.clone().unwrap().iter() {
                                if s == &n.name {
                                    found = true;
                                }
                            }
                        }
                    }
                    if !found {
                        updated.insert(0, n.clone());
                    }
                }

                // sort the available versions vector by semver
                available_versions.sort_unstable_by(compare_semver);
                log.mid(&format!(
                    "  {}",
                    "availble versions (use debug level to expand)"
                ));
                for version in available_versions.iter() {
                    log.debug(&format!("    {}", version));
                }
                log.lo("    suggested upgrade path");
                let mut upgrade_str = String::from(current_version.to_owned());
                let mut skip_range = String::from("");
                if current_version == "0.0.0" {
                    upgrade_str = "?".to_string();
                }
                for p in updated.iter() {
                    upgrade_str = upgrade_str + " -> " + &p.name;
                    if p.skip_range.is_some() {
                        skip_range = skip_range + " : " + &p.skip_range.clone().unwrap();
                    }
                }
                log.lo(&format!("    from {}", upgrade_str));
                if skip_range.len() > 0 {
                    log.lo(&format!("    skip_range {:#?} ", skip_range));
                }
            }
        }
    }
}

// utility sort by semver
fn compare_semver(a: &String, b: &String) -> Ordering {
    if a.contains(".v") && b.contains(".v") {
        let w = a.split(".v").nth(1).unwrap();
        let x = b.split(".v").nth(1).unwrap();
        let y = build_semver(w.to_string());
        let z = build_semver(x.to_string());
        return y.cmp(&z);
    }
    let w = a.split(".").nth(1).unwrap();
    let x = b.split(".").nth(1).unwrap();
    let y = build_semver(w.to_string());
    let z = build_semver(x.to_string());
    return y.cmp(&z);
}

// utility to build a more complex semver
fn build_semver(semver_str: String) -> semver::Version {
    let major: &str;
    let minor: &str;
    let mut patch: &str;
    let mut version: Version = Version {
        major: 0,
        minor: 0,
        patch: 0,
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    };
    let mut i = semver_str.split(".");
    if i.clone().count() > 1 {
        major = i.nth(0).unwrap();
        minor = i.nth(0).unwrap();
        patch = i.nth(0).unwrap();
        let mut tmp = String::from("");
        if patch.contains("-") {
            let mut x = patch.split("-");
            patch = x.nth(0).unwrap();
            tmp = x.nth(0).unwrap().to_string();
        }
        version.major = major.parse().unwrap();
        version.minor = minor.parse().unwrap();
        version.patch = patch.parse().unwrap();
        version.pre = Prerelease::new(&tmp).unwrap();
        version.build = BuildMetadata::EMPTY;
    } else {
        // this is some real kaka right here
        // in some of the catalog.json files we have incoherent semver (some use v some don't)
        // the wonderful world of devs with free range :)
        if semver_str.contains("v") {
            let mut n = semver_str.split("v");
            version.major = n.nth(1).unwrap().parse().unwrap();
        } else {
            version.major = i.nth(0).unwrap().parse().unwrap();
        }
    }
    version
}
