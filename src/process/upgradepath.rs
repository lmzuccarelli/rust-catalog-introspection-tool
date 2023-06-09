use crate::api::schema::*;
use crate::log::logging::*;
use crate::manifests::catalogs::*;
use std::fs;

// list all components in the current image index
pub async fn list_components(ctype: String, dir: String, filter: String) {
    let paths = fs::read_dir(&dir).unwrap();

    if filter != "all" {
        let dc = read_operator_catalog(dir + &"/".to_string() + &filter);
        log_hi(&filter);
        list_channel_info(dc.unwrap());
    } else {
        for path in paths {
            let entry = path.expect("could not resolve path entry");
            let dir_name = entry.path();
            let str_dir = dir_name.into_os_string().into_string().unwrap();
            let res = str_dir.split("/");
            let name = format!("{} => {}", ctype, res.into_iter().last().unwrap());
            let dc = read_operator_catalog(str_dir);
            log_hi(&name);
            list_channel_info(dc.unwrap());
        }
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
            log_mid(&format!("  channel name {}", x.name));
            for y in x.entries.unwrap() {
                log_ex(&format!("    bundle (in this channel) {}", y.name));
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
                    if !skip_range.contains(&sr) {
                        skip_range.insert(0, sr.clone());
                    }
                }
            }
            log_lo(&format!("    upgrade path  {:?}", current));
            if skip_range.len() > 0 {
                log_lo(&format!("    skip range {:?}", skip_range));
            }
        }
    }
}
