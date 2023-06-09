use flate2::read::GzDecoder;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
//use std::str;
use tar::Archive;

use crate::api::schema::*;
use crate::log::logging::*;

// get manifest async api call
pub async fn get_manifest(
    url: String,
    token: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);
    let body = client
        .get(url)
        .header("Accept", "application/vnd.oci.image.manifest.v1+json")
        .header("Content-Type", "application/json")
        .header("Authorization", header_bearer)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

// get each blob referred to by the vector in parallel 
// set by the PARALLEL_REQUESTS value
pub async fn get_blobs(url: String, token: String, layers: Vec<FsLayer>, dir: String) {
    const PARALLEL_REQUESTS: usize = 8;

    let inner_dir = &dir;
    let client = Client::new();
    let mut header_bearer: String = "Bearer ".to_owned();
    header_bearer.push_str(&token);

    // remove all duplicates in FsLayer
    let mut images = Vec::new();
    let mut seen = HashSet::new();
    for img in layers {
        if !seen.contains(&img.blob_sum) {
            seen.insert(img.blob_sum.clone());
            images.push(img.blob_sum);
        }
    }

    let fetches = stream::iter(images.into_iter().map(|blob| {
        let client = client.clone();
        let url = url.clone();
        let header_bearer = header_bearer.clone();
        async move {
            match client
                .get(url + &blob)
                .header("Authorization", header_bearer)
                .send()
                .await
            {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => {
                        let blob = blob.split(":").nth(1).unwrap();
                        fs::write(inner_dir.to_owned() + &blob, bytes.clone())
                            .expect("unable to write blob");
                        let msg = format!("writing blob {}", blob);
                        log_info(&msg);
                    }
                    Err(_) => {
                        let msg = format!("reading blob {}", &blob);
                        log_error(&msg);
                    }
                },
                Err(_) => {
                    let msg = format!("downloading blob {}", &blob);
                    log_error(&msg);
                }
            }
        }
    }))
    .buffer_unordered(PARALLEL_REQUESTS)
    .collect::<Vec<()>>();
    log_info("downloading blobs...");
    fetches.await;
}

// untar layers in directory denoted by parameter 'dir'
pub async fn untar_layers(dir: String) {
    // change to the blobs/sha256 directory
    let current_dir = env::current_dir().unwrap();
    env::set_current_dir(&dir).expect("could not set current directory");
    // read directory, iterate each file and untar
    let paths = fs::read_dir(".").unwrap();
    for path in paths {
        let entry = path.expect("could not resolve file entry");
        let file = entry.path();
        let tar_gz = File::open(file.clone()).expect("could not open file");
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);
        // should always be a sha256 string
        let tar_dir = file.into_os_string().into_string().unwrap();
        log_info(&format!("untarring file {} ", &tar_dir[2..8]));
        // we are really interested in either the configs or release-images directories
        match archive.unpack("../../cache/".to_string() + &tar_dir[2..8]) {
            Ok(arch) => arch,
            Err(error) => {
                let msg = format!("skipping this error : {} ", &error.to_string());
                log_warn(&msg);
            }
        };
    }
    env::set_current_dir(&current_dir).expect("could not set back to current directory");
}

// parse_image_index - best attempt to parse image index
pub fn parse_image_index(image: String) -> ImageReference {
    let mut img = image.split(":");
    let index = img.nth(0).unwrap();
    let mut img_ref = index.split("/");
    let ver = img.nth(0).unwrap();
    let ir = ImageReference {
        registry: img_ref.nth(0).unwrap().to_string(),
        namespace: img_ref.nth(0).unwrap().to_string(),
        name: img_ref.nth(0).unwrap().to_string(),
        version: ver.to_string(),
    };
    log_info(&format!("{:#?}", ir));
    ir
}

// contruct the manifest url
pub fn get_image_manifest_url(image_ref: ImageReference) -> String {
    // return a string in the form of (example below)
    // "https://registry.redhat.io/v2/redhat/certified-operator-index/manifests/v4.12";
    let mut url = String::from("https://");
    url.push_str(&image_ref.registry);
    url.push_str(&"/v2/");
    url.push_str(&image_ref.namespace);
    url.push_str(&"/");
    url.push_str(&image_ref.name);
    url.push_str(&"/");
    url.push_str(&"manifests/");
    url.push_str(&image_ref.version);
    url
}

// construct the blobs url
pub fn get_blobs_url(image_ref: ImageReference) -> String {
    // return a string in the form of (example below)
    // "https://registry.redhat.io/v2/redhat/certified-operator-index/blobs/";
    let mut url = String::from("https://");
    url.push_str(&image_ref.registry);
    url.push_str(&"/v2/");
    url.push_str(&image_ref.namespace);
    url.push_str("/");
    url.push_str(&image_ref.name);
    url.push_str(&"/");
    url.push_str(&"blobs/");
    url
}
