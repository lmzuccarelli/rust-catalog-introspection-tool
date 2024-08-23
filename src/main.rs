use clap::Parser;
use custom_logger::*;
use mirror_copy::*;
use std::fs;
use std::path::Path;
use std::process;
use tokio;

// define local modules
mod api;
mod config;
mod isc;
mod list;
mod operator;
mod upgradepath;

// use local modules
use api::schema::*;
use config::read::*;
use list::render::*;
use operator::collector::*;
use upgradepath::calculate::*;

// main entry point (use async)
#[tokio::main]
async fn main() -> Result<(), MirrorError> {
    let args = Cli::parse();

    let working_dir = args.working_dir.to_string();
    let lvl = args.loglevel.as_ref().unwrap();
    let skip_update = args.skip_update.as_ref().unwrap();
    let api_version = args.api_version.to_string();
    let output_dir = args.output_dir.to_string();

    let l = match lvl.as_str() {
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    let log = &Logging { log_level: l };

    // create artifacts directory
    let res = fs::create_dir_all(output_dir.clone());
    if res.is_err() {
        let err = MirrorError::new(&format!(
            "creating directory {} {}",
            output_dir.clone(),
            res.err().unwrap().to_string().to_lowercase()
        ));
        return Err(err);
    }

    match &args.command {
        Some(Commands::List {
            working_dir,
            catalog,
            operator,
        }) => {
            let res =
                render_list(log, working_dir.clone(), catalog.clone(), operator.clone()).await;
            if res.is_err() {
                let err = MirrorError::new(&format!(
                    "could not list catalog {} {}",
                    catalog,
                    res.err().unwrap().to_string().to_lowercase()
                ));
                return Err(err);
            }
        }
        None => {
            if args.config.is_none() {
                log.error("config file is required");
                process::exit(1);
            }

            // Parse the config serde_yaml::FilterConfiguration.
            let res_config = load_config(args.config.unwrap());
            if res_config.is_err() {
                let err = MirrorError::new(&format!(
                    "reading filter config {}",
                    res_config.err().unwrap().to_string().to_lowercase()
                ));
                return Err(err);
            }
            let config = res_config.unwrap();

            let res_fc = parse_yaml_config(config);
            if res_fc.is_err() {
                let err = MirrorError::new(&format!(
                    "parsing config {}",
                    res_fc.err().unwrap().to_string().to_lowercase()
                ));
                return Err(err);
            }
            let filter_config = res_fc.unwrap();

            log.debug(&format!("{:#?}", filter_config.operators));

            // verify catalog images
            let mut imgs: Vec<String> = vec![];
            for img in filter_config.catalogs.clone() {
                let i = img.split(":").nth(0).unwrap().to_string();
                if !imgs.contains(&i) {
                    imgs.insert(0, i);
                }
            }

            // quick check - catalog images must be greater than one
            if imgs.len() > 1 {
                log.error("catalog images are expected to be the same (except for versions)");
                process::exit(1);
            }

            // initialize the client request interface
            let reg_con = ImplRegistryInterface {};

            // check for catalog images
            if filter_config.catalogs.len() > 0 && !skip_update {
                let res = fs::create_dir_all(&working_dir);
                if res.is_err() {
                    let err = MirrorError::new(&format!(
                        "creating directory {}",
                        res.err().unwrap().to_string().to_lowercase()
                    ));
                    return Err(err);
                }
                // quickly convert to Operator struct
                let mut operators = vec![];
                for op in filter_config.catalogs.clone() {
                    let o = mirror_config::Operator {
                        catalog: op.clone(),
                        packages: None,
                    };
                    operators.insert(0, o);
                }
                let res =
                    get_operator_catalog(reg_con.clone(), log, working_dir.clone(), operators)
                        .await;
                if res.is_err() {
                    let err = MirrorError::new(&format!(
                        "creating directory {}",
                        res.err().unwrap().to_string().to_lowercase()
                    ));
                    return Err(err);
                }
            }

            list_components(
                log,
                api_version,
                working_dir,
                output_dir,
                filter_config.clone(),
            )
            .await;
        }
    }
    Ok(())
}
