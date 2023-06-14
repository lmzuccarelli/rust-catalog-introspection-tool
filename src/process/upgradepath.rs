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

    // iterate through the dc - look specifically for olm.channel schema
    for x in ch {
        if x.schema == "olm.channel" {
            if current_channel == x.name || current_channel == "all" {
                let mut current: Vec<String> = vec![];
                let mut skip_range: Vec<String> = vec![];
                let mut skip_tracker: Vec<String> = vec![];

                // entries contain all the relevant upgrade path info
                for y in x.entries.unwrap() {
                    // get the bundle semver
                    let mut semver_tmp = String::from("0.0.0");
                    if semver_tmp != "0.0.0" {}
                    if y.name.contains(".v") {
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
                        //log_mid(&format!("  channel name {}", x.name));
                        current.insert(0, y.name);
                        if y.replaces.is_some() {
                            let val = y.replaces.unwrap();
                            current.retain(|x| *x != val);
                        }
                        if y.skips.is_some() {
                            for skip in y.skips.unwrap() {
                                current.retain(|x| *x != skip);
                            }
                        }
                        if y.skip_range.is_some() {
                            let sr = y.skip_range.unwrap();
                            let hld = sr.split("<").nth(0).unwrap().to_string();
                            if !skip_tracker.contains(&hld) {
                                skip_tracker.insert(0, hld);
                                skip_range.insert(0, sr);
                            }
                        }
                    }
                }
                log.mid(&format!("  channel name {}", x.name));
                current.sort_unstable_by(compare_len_alpha);
                log.lo("    suggested upgrade path");
                let mut upgrade_str = String::from(current_version.to_owned());
                if current_version == "0.0.0" {
                    upgrade_str = "?".to_string();
                }
                for path in current {
                    upgrade_str = upgrade_str + " -> " + &path;
                }
                log.lo(&format!("    from {}", upgrade_str));
                if skip_range.len() > 0 {
                    skip_range.sort();
                    log.lo(&format!("    skip range {:?}", skip_range));
                }
            }
        }
    }
}

// utility sort by length first
fn compare_len_alpha(a: &String, b: &String) -> Ordering {
    // Sort by length from short to long first.
    let length_test = a.len().cmp(&b.len());
    if length_test == Ordering::Equal {
        // If same length, sort in alphanumeric order.
        return a.cmp(&b);
    }
    return length_test;
}

// utility to build a more complex semver
fn build_semver(semver_str: String) -> semver::Version {
    let p = semver_str.split(".").nth(2).unwrap();
    let tmp = p.split("-").nth(0).unwrap();
    let version = Version {
        major: semver_str.split(".").nth(0).unwrap().parse().unwrap(),
        minor: semver_str.split(".").nth(1).unwrap().parse().unwrap(),
        patch: tmp.parse().unwrap(),
        pre: Prerelease::new(p).unwrap(),
        build: BuildMetadata::EMPTY,
    };
    version
}
