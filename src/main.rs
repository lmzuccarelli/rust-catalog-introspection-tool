use clap::Parser;
use custom_logger::*;
use mirror_copy::*;
use std::fs;
use std::path::Path;
use std::process;
use tokio;

// define local modules
mod api;
mod calculate;
mod config;
mod isc;
mod operator;

// use local modules
use api::schema::*;
use calculate::upgradepath::*;
use config::read::*;
use operator::collector::*;

// main entry point (use async)
#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let cfg = args.config.to_string();
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
    if cfg == "" {
        log.error("config file is required");
        process::exit(1);
    }

    // create artifacts directory
    fs::create_dir_all(output_dir.clone()).expect("should create artifacts directory");

    // Parse the config serde_yaml::FilterConfiguration.
    let config = load_config(cfg).unwrap();
    let filter_config = parse_yaml_config(config).unwrap();
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
        fs::create_dir_all(&working_dir).expect("unable to create working directory");
        // quickly convert to Operator struct
        let mut operators = vec![];
        for op in filter_config.catalogs.clone() {
            let o = mirror_config::Operator {
                catalog: op.clone(),
                packages: None,
            };
            operators.insert(0, o);
        }
        get_operator_catalog(reg_con.clone(), log, working_dir.clone(), operators).await;
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
