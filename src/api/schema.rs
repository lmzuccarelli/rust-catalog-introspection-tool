// module api
use clap::Parser;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSchema {
    pub tag: String,
    pub name: String,
    pub architecture: String,
    pub schema_version: i64,
    pub history: Vec<History>,
    pub fs_layers: Vec<FsLayer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct History {
    #[serde(rename = "v1Compatibility")]
    pub v1compatibility: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsLayer {
    pub blob_sum: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i64,
    #[serde(rename = "issued_at")]
    pub issued_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub auths: Auths,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auths {
    #[serde(rename = "cloud.openshift.com")]
    pub cloud_openshift_com: CloudOpenshiftCom,
    #[serde(rename = "quay.io")]
    pub quay_io: QuayIo,
    #[serde(rename = "registry.connect.redhat.com")]
    pub registry_connect_redhat_com: RegistryConnectRedhatCom,
    #[serde(rename = "registry.redhat.io")]
    pub registry_redhat_io: RegistryRedhatIo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudOpenshiftCom {
    pub auth: String,
    pub email: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuayIo {
    pub auth: String,
    pub email: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryConnectRedhatCom {
    pub auth: String,
    pub email: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRedhatIo {
    pub auth: String,
    pub email: Option<String>,
}

/// rust-container-tool cli struct
#[derive(Parser, Debug)]
#[command(name = "rust-operator-upgradepath-tool")]
#[command(author = "Luigi Mario Zuccarelli <luzuccar@redhat.com>")]
#[command(version = "0.0.1")]
#[command(about = "Used to calcluate an upgrade path for a given (list) of operators", long_about = None)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config", default_value = "")]
    pub config: Option<String>,
}

/// config schema
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterConfig {
    #[serde(rename = "kind")]
    pub kind: String,

    #[serde(rename = "apiVersion")]
    pub api_version: String,

    #[serde(rename = "catalog")]
    pub catalog: String,

    #[serde(rename = "packages")]
    pub operators: Option<Vec<Operator>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Operator {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "channel")]
    pub channel: Option<String>,

    #[serde(rename = "fromVersion")]
    pub from_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Catalog {
    #[serde(rename = "overview")]
    pub overview: serde_json::Value,
}

// DeclarativeConfig this updates the existing dclrcfg
#[derive(Serialize, Deserialize, Debug)]
pub struct DeclarativeEntries {
    #[serde(rename = "entries")]
    pub entries: Option<Vec<ChannelEntry>>,
    pub channel: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeclarativeConfig {
    #[serde(rename = "schema")]
    pub schema: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "package")]
    pub package: Option<String>,

    #[serde(rename = "relatedImages")]
    pub related_images: Option<Vec<RelatedImage>>,

    #[serde(rename = "defaultChannel")]
    pub default_channel: Option<String>,

    #[serde(rename = "description")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelatedImage {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "image")]
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    #[serde(rename = "schema")]
    pub schema: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "defaultChannel")]
    pub default_channel: Option<String>,
}

// Channel used in parsing channel data
#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    #[serde(rename = "schema")]
    pub schema: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "package")]
    pub package: Option<String>,

    #[serde(rename = "entries")]
    pub entries: Option<Vec<ChannelEntry>>,
}

// ChannelEntry used in the Channel struct
#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelEntry {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "replaces")]
    pub replaces: Option<String>,

    #[serde(rename = "skips")]
    pub skips: Option<Vec<String>>,

    #[serde(rename = "skipRange")]
    pub skip_range: Option<String>,
}

// Bundle specifies all metadata and data of a bundle object.
// Top-level fields are the source of truth, i.e. not CSV values.
//
// Notes:
//   - Any field slice type field or type containing a slice somewhere
//     where two types/fields are equal if their contents are equal regardless
//     of order must have a `hash:"set"` field tag for bundle comparison.
//   - Any fields that have a `json:"-"` tag must be included in the equality
//     evaluation in bundlesEqual().
#[derive(Serialize, Deserialize, Debug)]
pub struct Bundle {
    #[serde(rename = "schema")]
    pub schema: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "package")]
    pub package: String,

    #[serde(rename = "image")]
    pub image: String,

    #[serde(rename = "relatedImages")]
    pub related_images: Vec<RelatedImage>,
    // These fields are present so that we can continue serving
    // the GRPC API the way packageserver expects us to in a
    // backwards-compatible way. These are populated from
    // any `olm.bundle.object` properties.
    //
    // These fields will never be persisted in the bundle blob as
    // first class fields.

    //CsvJSON string   `json:"-"`
    //Objects []string `json:"-"`
}

// ImageReference
#[derive(Debug, Clone)]
pub struct ImageReference {
    pub registry: String,
    pub namespace: String,
    pub name: String,
    pub version: String,
}
