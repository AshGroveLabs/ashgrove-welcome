use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PackCategory,

    #[serde(default)]
    pub host_packages: Vec<String>,

    #[serde(default)]
    pub flatpaks: Vec<String>,

    #[serde(default)]
    pub distrobox_packages: Vec<String>,

    #[serde(default)]
    pub requires_reboot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackCategory {
    Development,
    Gaming,
    Productivity,
    CloudSync,
    Ecosystem,
    System,
}
