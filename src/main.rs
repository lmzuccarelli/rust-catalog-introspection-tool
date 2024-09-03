use clap::Parser;
use custom_logger::*;
use mirror_copy::*;
use mirror_error::MirrorError;
use mirror_utils::fs_handler;
use std::fs;
use std::process;
use tokio;

// define local modules
mod api;
mod batch;
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

    let lvl = args.loglevel.as_ref().unwrap();

    let l = match lvl.as_str() {
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    let log = &Logging { log_level: l };

    match &args.command {
        Some(Commands::List {
            working_dir,
            catalog,
            operator,
        }) => {
            let res =
                render_list(log, working_dir.clone(), catalog.clone(), operator.clone()).await;
            if res.is_err() {
                log.error(&format!(
                    "[main] {} {}",
                    catalog,
                    res.err().unwrap().to_string().to_lowercase()
                ));
                process::exit(1);
            }
        }
        Some(Commands::Update {
            working_dir,
            config_file,
        }) => {
            // Parse the config serde_yaml::FilterConfiguration.
            let res_config = load_config(config_file.to_string()).await?;
            let res_fc = parse_yaml_config(res_config)?;

            log.debug(&format!("{:#?}", res_fc.operators.clone()));

            // verify catalog images
            let mut imgs: Vec<String> = vec![];
            for img in res_fc.catalogs.clone() {
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
            if res_fc.catalogs.len() > 0 {
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
                for op in res_fc.catalogs.clone() {
                    let o = mirror_config::Operator {
                        catalog: op.clone(),
                        packages: None,
                    };
                    operators.insert(0, o);
                }
                let res = get_operator_catalog(
                    reg_con.clone(),
                    log,
                    working_dir.clone(),
                    false,
                    true,
                    operators,
                )
                .await;
                if res.is_err() {
                    let err = MirrorError::new(&format!(
                        "creating directory {}",
                        res.err().unwrap().to_string().to_lowercase()
                    ));
                    return Err(err);
                }
            }
        }
        Some(Commands::Upgradepath {
            config_file,
            working_dir,
            output_dir,
            api_version,
        }) => {
            // create artifacts directory
            fs_handler(output_dir.clone(), "create_dir", None).await?;
            // Parse the config serde_yaml::FilterConfiguration.
            let res_config = load_config(config_file.to_string()).await?;
            let res_fc = parse_yaml_config(res_config)?;

            process_upgradepath(
                log,
                api_version.to_string(),
                working_dir.to_string(),
                output_dir.to_string(),
                res_fc.clone(),
            )
            .await;
        }
        None => {
            log.error(
                "please ensure you have selected the correct sub command use --help for assistence",
            );
            process::exit(1);
        }
    }
    Ok(())
}
