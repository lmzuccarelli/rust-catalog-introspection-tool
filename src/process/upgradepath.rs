use crate::api::schema::*;
use crate::log::logging::*;
use crate::manifests::catalogs::*;
use std::cmp::*;
//use std::fs;

// list all components in the current image index
pub async fn list_components(dir: String, filter: FilterConfig) {
    //let paths = fs::read_dir(&dir).unwrap();

    for operator in filter.operators {
        let dc = read_operator_catalog(dir.to_string() + &"/".to_string() + &operator.name);
        log_hi(&format!("operator '{}'", operator.name));
        list_channel_info(dc.unwrap());
    }
}

// iterate through object and display values
pub fn list_channel_info(dc: serde_json::Value) {
    let dc: Vec<Channel> = match serde_json::from_value(dc.clone()) {
        Ok(val) => val,
        Err(error) => panic!("error {}", error),
    };

    for x in dc {
        if x.schema == "olm.channel" {
            let mut current: Vec<String> = vec![];
            let mut skip_range: Vec<String> = vec![];
            let mut skip_tracker: Vec<String> = vec![];
            //let mut semver_tracker: Vec<String> = vec![];
            log_mid(&format!("  channel name {}", x.name));
            for y in x.entries.unwrap() {
                log_ex(&format!("    bundle {}", y.name));
                //semver_tracker.insert(0, y.name.split(".v").nth(1).unwrap().to_string());
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
            current.sort_unstable_by(compare_len_alpha);
            //semver_tracker.sort_unstable_by(compare_len_reverse_alpha);
            log_lo(&format!("    upgrade path  {:?}", current));
            //log_lo(&format!("    semver  {:?}", semver_tracker));
            if skip_range.len() > 0 {
                skip_range.sort();
                log_lo(&format!("    skip range {:?}", skip_range));
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
