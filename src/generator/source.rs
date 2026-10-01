use std::path::{Path, PathBuf};

use super::error::BuildError;

pub(super) struct SourcePaths {
    pub(super) content: PathBuf,
    pub(super) templates: PathBuf,
    pub(super) assets: PathBuf,
}

pub(super) fn validate_source(source: &Path) -> Result<SourcePaths, BuildError> {
    if !source.exists() {
        return Err(BuildError::SourceDNE(source.to_path_buf()));
    }

    let content = source.join("content");
    let templates = source.join("templates");
    let assets = source.join("assets");

    if !content.exists() {
        return Err(BuildError::ContentDNE(content));
    } else if !templates.exists() {
        return Err(BuildError::TemplatesDNE(templates));
    } else if !assets.exists() {
        return Err(BuildError::AssetsDNE(assets));
    }

    Ok(SourcePaths {
        content,
        templates,
        assets,
    })
}
