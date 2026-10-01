mod asset;
mod content;
mod error;
mod front_matter;
mod output;
mod site;
mod source;

use std::path::Path;

use asset::copy_assets;
use content::build_content;
pub use error::{AssetError, BuildError, ConfigError, ContentError, FrontMatterError};
use output::prepare_output;
use site::load_site_config;
use source::validate_source;

pub fn build_site(source: &Path, output: &Path) -> Result<BuildReport, BuildError> {
    let source_paths = validate_source(source)?;
    let output_paths = prepare_output(output)?;

    let site = load_site_config(source)?;
    let pages_written = build_content(
        &source_paths.content,
        &source_paths.templates,
        &output_paths.root,
        &site,
    )?;

    let assets_copied = copy_assets(&source_paths.assets, &output_paths.assets)?;

    Ok(BuildReport {
        pages_written,
        assets_copied,
    })
}

pub struct BuildReport {
    pub pages_written: usize,
    pub assets_copied: usize,
}
