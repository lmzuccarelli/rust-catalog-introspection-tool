use clap::Parser;
use std::fs;
use std::path::Path;
use tokio;

// define modules
mod api;
mod auth;
mod config;
mod image;
mod log;
mod manifests;
mod process;

use api::schema::*;
use auth::credentials::*;
use config::read::*;
use image::copy::*;
use log::logging::*;
use manifests::catalogs::*;
use process::upgradepath::*;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let filter = args.filter.as_ref().unwrap().to_string();
    let cfg = args.config.as_ref().unwrap().to_string();

    log_info(&format!(
        "rust-container-tools {} {} {}",
        cfg, args.image, filter
    ));

    let img_ref = parse_image_index(args.image);

    // Parse the config serde_yaml::ImageSetConfig.
    if cfg != "" {
        let config = load_config(cfg).unwrap();
        let isc = parse_yaml_config(config).unwrap();
        log_debug(&format!("{:#?}", isc.mirror.platform));
    }

    let manifest_json = get_manifest_json_file(img_ref.name.clone(), img_ref.version.clone());
    let working_dir_blobs = get_blobs_dir(img_ref.name.clone(), img_ref.version.clone());
    let working_dir_cache = get_cache_dir(img_ref.name.clone(), img_ref.version.clone());

    // check if the directory exists
    if !Path::new(&working_dir_blobs).exists() {
        let token = get_token(img_ref.registry.clone()).await;
        // use token to get manifest
        let manifest_url = get_image_manifest_url(img_ref.clone());
        let manifest = get_manifest(manifest_url.clone(), token.clone())
            .await
            .unwrap();

        // create the full path
        fs::create_dir_all(working_dir_blobs.clone()).expect("unable to create directory");
        fs::write(manifest_json, manifest.clone()).expect("unable to write file");
        let res = parse_json_manifest(manifest).unwrap();
        let blobs_url = get_blobs_url(img_ref.clone());
        get_blobs(blobs_url, token, res.fs_layers, working_dir_blobs.clone()).await;
        log_info("completed image index download");
    } else {
        log_info("catalog index exists nothing to do");
    }
    // check if the cache directory exists
    if !Path::new(&working_dir_cache).exists() {
        // create the cache directory
        fs::create_dir_all(&working_dir_cache).expect("unable to create directory");
        untar_layers(working_dir_blobs.clone()).await;
        log_info("completed untar of layers");
    } else {
        log_info("cache exists nothing to do");
    }
    if args.action == "list" {
        let dir = find_dir(working_dir_cache.clone(), "configs".to_string()).await;
        log_info(&format!("full path for directory 'configs' {} ", &dir));
        if dir != "" {
            list_components("operator".to_string(), dir, filter).await;
        } else {
            log_error("configs directory not found");
        }
    }
}

// utility functions
// get_manifest_json
fn get_manifest_json_file(name: String, version: String) -> String {
    let mut file = String::from("working-dir/");
    file.push_str(&name);
    file.push_str(&"/");
    file.push_str(&version);
    file.push_str(&"/");
    file.push_str(&"manifest.json");
    file
}

// get_blobs_dir
fn get_blobs_dir(name: String, version: String) -> String {
    let mut file = String::from("working-dir/");
    file.push_str(&name);
    file.push_str(&"/");
    file.push_str(&version);
    file.push_str(&"/");
    file.push_str(&"blobs/sha256/");
    file
}

// get_cache_dir
fn get_cache_dir(name: String, version: String) -> String {
    let mut file = String::from("working-dir/");
    file.push_str(&name);
    file.push_str(&"/");
    file.push_str(&version);
    file.push_str(&"/");
    file.push_str(&"cache");
    file
}
