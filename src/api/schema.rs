// module api
use clap::Parser;
use serde_derive::Deserialize;
use serde_derive::Serialize;

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = "rust-operator-upgradepath-tool")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.0.1")]
#[command(about = "Used to calcluate an upgrade path (heuristic approach) for a given (list) of operators", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: Option<String>,

    #[arg(
        value_enum,
        short,
        long,
        value_name = "loglevel",
        default_value = "info",
        help = "set the log level [possible values: info, debug, trace]"
    )]
    pub loglevel: Option<String>,

    #[arg(
        short,
        long,
        value_name = "skip-update",
        default_value = "false",
        help = "if set will skip the catalog update check"
    )]
    pub skip_update: Option<bool>,
}

/// config schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterConfig {
    #[serde(rename = "kind")]
    pub kind: String,

    #[serde(rename = "apiVersion")]
    pub api_version: String,

    #[serde(rename = "catalogs")]
    pub catalogs: Vec<String>,

    #[serde(rename = "packages")]
    pub operators: Option<Vec<FilterOperator>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterOperator {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "channel")]
    pub channel: Option<String>,

    #[serde(rename = "fromVersion")]
    pub from_version: Option<String>,
}
