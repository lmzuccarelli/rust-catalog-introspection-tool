use custom_logger::*;
use mirror_catalog::*;
use mirror_catalog_index::find_dir;
use mirror_error::MirrorError;
use walkdir::WalkDir;

pub async fn render_list(
    log: &Logging,
    dir: String,
    catalog: String,
    operator: Option<String>,
) -> Result<(), MirrorError> {
    log.info(&format!("list catalog {}", catalog));
    // list the operators found in the filter

    let index_dir = catalog.replace(":", "/");
    let catalog_dir = format!("{}/{}/{}", dir.clone(), &index_dir, "/amd64/cache/");

    if operator.is_none() {
        let result = WalkDir::new(&catalog_dir);
        println!("");
        println!("\x1b[05C{}", "\x1b[1;97mOPERATORS\x1b[0m");
        println!("\x1b[05C\x1b[1;97m-----------------------------------------\x1b[0m");
        println!("");
        for file in result.into_iter() {
            // iterate through each operator in the filterconfig
            let res = file.as_ref().clone();
            if res.is_ok() {
                let check = res.as_ref().unwrap();
                if check.path().is_dir() {
                    let f = file.unwrap().clone().path().display().to_string();
                    if f.contains("/configs/") && !f.contains("/updated-configs") {
                        println!(
                            "\x1b[05C\x1b[0;97m{}\x1b[0m",
                            f.split("/configs/").nth(1).unwrap()
                        );
                    }
                }
            } else {
                let err = MirrorError::new(&format!(
                    "reading file {}",
                    res.err().unwrap().to_string().to_lowercase()
                ));
                return Err(err);
            }
        }
    } else {
        let config_dir = find_dir(log, catalog_dir.clone(), "configs".to_string()).await;
        let operator_file = format!(
            "{}/{}/updated-configs/",
            config_dir,
            operator.as_ref().unwrap()
        );
        let dc_map = DeclarativeConfig::get_declarativeconfig_map(operator_file);
        let mut default_channel: String = String::new();
        let keys = dc_map.keys();
        println!("");
        println!("\x1b[05C\x1b[1;97mOPERATOR\x1b[0m\x1b[42C\x1b[1;97mCHANNELS\x1b[0m\x1b[22C\x1b[1;97mBUNDLES\x1b[0m");
        println!("\x1b[05C\x1b[1;97m------------------------------------------------  ----------------------------  ------------------------------------------\x1b[0m");
        for k in keys.clone() {
            if k.contains("olm.package") {
                let pkg = dc_map.get(k).unwrap();
                default_channel = pkg.default_channel.clone().unwrap();
                break;
            }
        }
        println!("\x1b[05C\x1b[1;97m{}\x1b[0m\x1b[1A", operator.unwrap());
        for k in keys {
            if k.contains("olm.channel") {
                let channel = dc_map.get(k).unwrap();
                let name = channel.name.as_ref().unwrap().to_string();
                if name == default_channel {
                    println!(
                        "\x1b[55C\x1b[1;94m{}\x1b[0m\x1b[1A",
                        channel.name.as_ref().unwrap()
                    );
                } else {
                    println!("\x1b[55C{}\x1b[1A", channel.name.as_ref().unwrap());
                }
                for e in channel.entries.as_ref().unwrap().iter() {
                    println!("\x1b[85C\x1b[0;97m{}\x1b[0m", e.name);
                }
                println!("");
            }
        }
    }
    Ok(())
}
