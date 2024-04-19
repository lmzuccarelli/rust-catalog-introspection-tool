use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IscV3Alpha1 {
    pub api_version: String,
    pub operators: Vec<Catalog>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub packages: Vec<Package>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub bundles: Vec<Bundle>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
}

impl IscV3Alpha1 {
    pub fn new() -> Self {
        IscV3Alpha1 {
            api_version: "v3alpha1".to_string(),
            operators: vec![],
        }
    }

    pub fn to_yaml(&self) -> String {
        let mut body = "".to_string();
        let yaml = format!(
            "\n---\napiVersion: mirror.openshift/{}
kind: ImageSetConfiguration
metadata:
  name: ImageSetConfiguration
  annotations: 
    autogenerated: 'rust-catalog-introspection-tool'
mirror:
  operators:
",
            self.api_version
        );
        for ops in self.operators.iter() {
            body += &format!("  - packages:");
            for pkg in ops.packages.iter() {
                body += &format!("\n      - bundles:");
                pkg.bundles.iter().for_each(|b| {
                    body += &format!("\n        - name: {}", b.name);
                });
            }
        }
        let all = yaml + &body;
        all
    }
}
