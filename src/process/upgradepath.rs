use crate::api::schema::*;
use crate::log::logging::*;
use crate::manifests::catalogs::*;
//use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use semver::Version;
use std::cmp::*;

// list all components in the current image index
pub async fn list_components(dir: String, filter: FilterConfig) {
    for operator in filter.operators {
        let dc = read_operator_catalog(dir.to_string() + &"/".to_string() + &operator.name);
        list_channel_info(dc.unwrap(), operator);
    }
}

// iterate through object and display values
pub fn list_channel_info(dc: serde_json::Value, filter: Operator) {
    let dc: Vec<Channel> = match serde_json::from_value(dc.clone()) {
        Ok(val) => val,
        Err(error) => panic!("error {}", error),
    };

    // check to see if filter.from_version is valid (or empty)
    let mut current_semver = Version::parse("0.0.0").unwrap();
    let mut current_version = String::from("0.0.0");

    if filter.from_version.is_some() {
        current_version = filter.from_version.unwrap();
        current_semver = Version::parse(&current_version).unwrap();
    }

    // check to see if filter.channel is valid (or empty)
    let mut current_channel = String::from("all");
    if filter.channel.is_some() {
        current_channel = filter.channel.unwrap();
    }

    log_ex(&format!("filter version {:?}", current_version));
    log_ex(&format!("filter semver  {:?}", current_semver));
    log_ex(&format!("filter channel {:?}", current_channel));
    log_hi(&format!("operator '{}'", filter.name));

    // iterate through the dc - look specifically for olm.channel schema
    for x in dc {
        if x.schema == "olm.channel" {
            if current_channel == x.name || current_channel == "all" {
                let mut current: Vec<String> = vec![];
                let mut skip_range: Vec<String> = vec![];
                let mut skip_tracker: Vec<String> = vec![];

                // entries contain all the relevant upgrade path info
                for y in x.entries.unwrap() {
                    // get the bundle semver
                    let semver_tmp = y.name.split(".v").nth(1).unwrap();
                    let bundle_semver = Version::parse(semver_tmp).unwrap();

                    let res = current_semver.cmp(&bundle_semver);
                    if res != Ordering::Greater {
                        log_mid(&format!("  channel name {}", x.name));
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
                current.sort_unstable_by(compare_len_alpha);
                if current.len() > 0 {
                    log_lo(&format!("    upgrade path  {:?}", current));
                }
                if skip_range.len() > 0 {
                    skip_range.sort();
                    log_lo(&format!("    skip range {:?}", skip_range));
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
