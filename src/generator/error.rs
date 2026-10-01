use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("source folder {0} does not exist")]
    SourceDNE(PathBuf),

    #[error("content folder {0} does not exist")]
    ContentDNE(PathBuf),

    #[error("templates folder {0} does not exist")]
    TemplatesDNE(PathBuf),

    #[error("assets folder {0} does not exist")]
    AssetsDNE(PathBuf),

    #[error("could not create output directory {path}")]
    CreateOutputDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed loading site configuration")]
    Config(#[from] ConfigError),

    #[error("failed generating content")]
    Content(#[from] ContentError),

    #[error("failed copying assets")]
    Assets(#[from] AssetError),

    #[error("failed staging site")]
    Staging(#[from] StagingError),

    #[error("failed planning file generation")]
    Plan(#[from] PlanError),
}

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("failed to traverse the content directory")]
    WalkDir(#[from] walkdir::Error),

    #[error("failed to read {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid front matter in {path}")]
    FrontMatter {
        path: PathBuf,
        #[source]
        source: FrontMatterError,
    },

    #[error("failed to load template {template:?} for {page}")]
    LoadTemplate {
        page: PathBuf,
        template: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("failed to render template {template:?} for {page}")]
    RenderTemplate {
        page: PathBuf,
        template: String,
        #[source]
        source: minijinja::Error,
    },

    #[error("failed to create page directory {path}")]
    CreatePageDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("content file {path} is outside content directory {content_root}")]
    OutsideContentDirectory {
        path: PathBuf,
        content_root: PathBuf,
    },

    #[error("unsupported content file {path}; expected a Markdown file")]
    UnsupportedContentFile { path: PathBuf },

    #[error("cannot determine an output path for {path}")]
    InvalidContentPath { path: PathBuf },

    #[error("failed to write page {path}")]
    WritePage {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum FrontMatterError {
    #[error("document does not begin with a `---` front-matter delimeter")]
    MissingOpeningDelimeter,

    #[error("front matter has no closing `---` delimeter")]
    MissingClosingDelimeter,

    #[error("invalid YAML front matter")]
    InvalidYaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read site configuration at {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid site configuration at {path}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("failed to traverse the assets directory")]
    WalkDir(#[from] walkdir::Error),

    #[error("asset {path} is outside the asset directory {assets_root}")]
    OutsideAssetsDirectory { path: PathBuf, assets_root: PathBuf },

    #[error("failed to create asset directory {path}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to copy asset from {source_path} to {destination}")]
    Copy {
        source_path: PathBuf,
        destination: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("unsupported asset type at {path}")]
    UnsupportedFileType { path: PathBuf },
}

#[derive(Debug, Error)]
pub enum StagingError {
    #[error("failed to publish site")]
    Publish(#[from] std::io::Error),

    #[error("failed to create staging directory")]
    CreateStagingDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum PlanError {
    #[error("failed to traverse a source directory")]
    WalkDir(#[from] walkdir::Error),

    #[error(transparent)]
    Content(#[from] ContentError),

    #[error("asset {path} is outside the asset directory {assets_root}")]
    OutsideAssetsDirectory { path: PathBuf, assets_root: PathBuf },

    #[error("unsupported asset type at {path}")]
    UnsupportedAssetType { path: PathBuf },

    #[error(
        "output collision at {destination}: {existing_source:?} conflicts with {incoming_source}"
    )]
    Collision {
        destination: PathBuf,
        existing_source: Option<PathBuf>,
        incoming_source: PathBuf,
    },
}
