mod asset;
mod content;
mod error;
mod front_matter;
mod output;
mod plan;
mod site;
mod source;

use std::path::Path;

use asset::copy_assets;
use content::build_content;
pub use error::{AssetError, BuildError, ConfigError, ContentError, FrontMatterError};
use output::{StagedOutput, prepare_output};
use site::load_site_config;
use source::validate_source;
use plan::BuildPlan;


pub fn build_site(source: &Path, output: &Path) -> Result<BuildReport, BuildError> {
    let source_paths = validate_source(source)?;
    let site = load_site_config(source)?;

    let plan = BuildPlan::discover(&source_paths)?;

    let staged_output = StagedOutput::new(output)?;
    let output_paths = prepare_output(staged_output.path())?;

    let pages_written = build_content(&plan.pages, &source_paths.templates, &output_paths.root, &site)?;

    let assets_copied = copy_assets(&plan.assets, &output_paths.assets)?;

    staged_output.publish()?;

    Ok(BuildReport {
        pages_written,
        assets_copied,
    })
}

pub struct BuildReport {
    pub pages_written: usize,
    pub assets_copied: usize,
}
