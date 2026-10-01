use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use super::error::ConfigError;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct SiteConfig {
    title: String,
    description: String,
    base_url: String,

    #[serde(default = "default_language")]
    language: String,

    author: Option<Author>,

    #[serde(default)]
    navigation: Vec<NavigationItem>,

    #[serde(default)]
    extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Author {
    name: String,
    email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NavigationItem {
    label: String,
    url: String,
}

pub(super) fn load_site_config(source: &Path) -> Result<SiteConfig, ConfigError> {
    let path = source.join("site.toml");

    let contents = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;

    toml::from_str(&contents).map_err(|source| ConfigError::Parse { path, source })
}

fn default_language() -> String {
    String::from("en")
}
