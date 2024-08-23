// module api
use clap::{Parser, Subcommand};
use serde_derive::Deserialize;
use serde_derive::Serialize;

/// rust-container-tool cli struct
#[derive(Parser)]
#[command(name = "rust-operator-introspection-tool")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.0.1")]
#[command(about = "Used to calcluate an upgrade path (heuristic approach) for a given (list) of operators and generate appropriate imagesetconfig yaml files", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: Option<String>,

    #[arg(
        value_enum,
        short,
        long,
        value_name = "loglevel",
        default_value = "info",
        help = "Set the log level [possible values: info, debug, trace]"
    )]
    pub loglevel: Option<String>,

    #[arg(
        short,
        long,
        value_name = "skip-update",
        default_value = "false",
        help = "If set will skip the catalog update check"
    )]
    pub skip_update: Option<bool>,

    #[arg(
        short,
        long,
        value_name = "working-dir",
        default_value = "",
        help = "Sets the working-dir, used to share existing caches with other catalog tooling"
    )]
    pub working_dir: String,

    #[arg(
        short,
        long,
        value_name = "api-version",
        default_value = "v2alpha1",
        help = "Sets the api version for the imagesetconfig"
    )]
    pub api_version: String,

    #[arg(
        short,
        long,
        value_name = "output-dir",
        default_value = "artifacts",
        help = "The directory to output the auto-generated imagesetconfig to"
    )]
    pub output_dir: String,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List subcommand (lists operators in a catalog)
    List {
        #[arg(
            short,
            long,
            value_name = "working-dir",
            help = "Sets the working-dir, used to share existing caches with other catalog tooling (required)"
        )]
        working_dir: String,

        #[arg(
            short,
            long,
            value_name = "catalog",
            help = "Lists all the operators in the specified catalog (required)"
        )]
        catalog: String,

        #[arg(
            short,
            long,
            value_name = "operator",
            help = "Filter the list with a specific operator"
        )]
        operator: Option<String>,
    },
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
