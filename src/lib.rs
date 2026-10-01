use std::{
    collections::BTreeMap, ffi::OsStr, fs::{self, create_dir_all}, path::{Path, PathBuf}
};

use minijinja::{AutoEscape, Environment, Value, context, path_loader};
use pulldown_cmark::{Parser, html};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

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
    UnsupportedContentFile {
        path: PathBuf,
    },

    #[error("cannot determine an output path for {path}")]
    InvalidContentPath {
        path: PathBuf,
    },

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
    OutsideAssetsDirectory {
        path: PathBuf,
        assets_root: PathBuf,
    },

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
    UnsupportedFileType {
        path: PathBuf,
    },
}

pub fn build_site(source: &Path, output: &Path) -> Result<BuildReport, BuildError> {
    let source_paths = validate_source(source)?;
    let output_paths = prepare_output(output)?;

    let site = load_site_config(source)?;
    let templates = create_template_environment(&source_paths.templates);

    let pages_written =
        build_content(&source_paths.content, &output_paths.root, &templates, &site)?;

    let assets_copied = copy_assets(&source_paths.assets, &output_paths.assets)?;

    Ok(BuildReport {
        pages_written,
        assets_copied,
    })
}

struct SourcePaths {
    content: PathBuf,
    templates: PathBuf,
    assets: PathBuf,
}

fn validate_source(source: &Path) -> Result<SourcePaths, BuildError> {
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

struct OutputPaths {
    root: PathBuf,
    assets: PathBuf,
}

fn create_output_directory(path: &Path) -> Result<(), BuildError> {
    std::fs::create_dir_all(path).map_err(|source| BuildError::CreateOutputDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn prepare_output(output: &Path) -> Result<OutputPaths, BuildError> {
    let paths = OutputPaths {
        root: output.to_path_buf(),
        assets: output.join("assets"),
    };

    create_output_directory(&paths.root)?;
    create_output_directory(&paths.assets)?;

    Ok(paths)
}

#[derive(Debug, Serialize, Deserialize)]
struct SiteConfig {
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

fn default_language() -> String {
    String::from("en")
}

fn load_site_config(source: &Path) -> Result<SiteConfig, ConfigError> {
    let path = source.join("site.toml");

    let contents = std::fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;

    toml::from_str(&contents).map_err(|source| ConfigError::Parse { path, source })
}

fn create_template_environment(templates: &Path) -> Environment<'static> {
    let mut environment = Environment::new();

    environment.set_loader(path_loader(templates));

    environment.set_auto_escape_callback(|name| {
        if name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });

    environment
}

#[derive(Debug, Serialize)]
struct TemplatePage {
    title: String,
    description: String,
    content: Value,
}

fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();

    html::push_html(&mut html_output, parser);
    html_output
}

fn render_page(
    environment: &Environment<'_>,
    front_matter: FrontMatter,
    markdown: &str,
) -> Result<String, minijinja::Error> {
    let template = environment.get_template(&front_matter.template)?;
    let content_html = render_markdown(markdown);

    let page = TemplatePage {
        title: front_matter.title,
        description: front_matter.description,
        content: Value::from_safe_string(content_html),
    };

    template.render(context! {
        page => page
    })
}

fn build_content(
    content: &Path,
    output: &Path,
    templates: &Environment<'_>,
    site: &SiteConfig,
) -> Result<usize, ContentError> {
    let mut pages_written = 0;

    for entry in WalkDir::new(content) {
        let entry = entry?;
        let source_path = entry.path();

        if !source_path.is_file()
            || source_path.extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }

        let document =
            std::fs::read_to_string(source_path).map_err(|source| ContentError::Read {
                path: source_path.to_path_buf(),
                source,
            })?;

        let (front_matter, markdown) =
            parse_document(&document).map_err(|source| ContentError::FrontMatter {
                path: source_path.to_path_buf(),
                source,
            })?;

        let html = render_markdown(markdown);

        let template_name = front_matter.template.clone();
        let template = templates.get_template(&template_name).map_err(|source| {
            ContentError::LoadTemplate {
                page: source_path.to_path_buf(),
                template: template_name.clone(),
                source,
            }
        })?;

        let rendered = template
            .render(minijinja::context! {
                site => site,
                page => minijinja::context! {
                    title => front_matter.title,
                    description => front_matter.description,
                    content => minijinja::Value::from_safe_string(html),
                },
            })
            .map_err(|source| ContentError::RenderTemplate {
                page: source_path.to_path_buf(),
                template: template_name,
                source,
            })?;

        let relative_output = page_output_path(content, source_path)?;
        write_page(output, &relative_output, &rendered)?;

        pages_written += 1;
    }

    Ok(pages_written)
}

fn page_output_path(content_root: &Path, source_path: &Path) -> Result<PathBuf, ContentError> {
    let relative = source_path
        .strip_prefix(content_root)
        .map_err(|_| ContentError::OutsideContentDirectory {
            path: source_path.to_path_buf(),
            content_root: content_root.to_path_buf(),
        })?;

    if relative.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(ContentError::UnsupportedContentFile {
            path: source_path.to_path_buf(),
        });
    }

    let file_stem = relative.file_stem().ok_or_else(|| {
        ContentError::InvalidContentPath {
            path: source_path.to_path_buf(),
        }
    })?;

    if file_stem == OsStr::new("index") {
        Ok(relative.with_extension("html"))
    } else {
        let mut output = relative.with_extension("");
        output.push("index.html");
        Ok(output)
    }
}

fn write_page(output: &Path, relative_path: &Path, html: &str) -> Result<(), ContentError> {
    let destination = output.join(relative_path);

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ContentError::CreatePageDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    std::fs::write(&destination, html).map_err(|source| ContentError::WritePage {
        path: destination,
        source,
    })
}

fn copy_assets(assets_root: &Path, output_assets: &Path) -> Result<usize, AssetError> {
    create_asset_directory(output_assets)?;

    let mut files_copied = 0;

    for entry in WalkDir::new(assets_root).follow_links(false) {
        let entry = entry?;
        let source_path = entry.path();

        let relative_path = source_path
            .strip_prefix(assets_root)
            .map_err(|_| AssetError::OutsideAssetsDirectory {
                path: source_path.to_path_buf(),
                assets_root: assets_root.to_path_buf(),
            })?;

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let destination = output_assets.join(relative_path);
        let file_type = entry.file_type();

        if file_type.is_dir() {
            create_asset_directory(&destination)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination.parent() {
                create_asset_directory(parent)?;
            }

            fs::copy(source_path, &destination).map_err(|source| {
                AssetError::Copy {
                    source_path: source_path.to_path_buf(),
                    destination,
                    source,
                }
            })?;

            files_copied += 1;
        } else {
            return Err(AssetError::UnsupportedFileType {
                path: source_path.to_path_buf(),
            });
        }
    }

    Ok(files_copied)
}

fn create_asset_directory(path: &Path) -> Result<(), AssetError> {
    fs::create_dir_all(path).map_err(|source| {
        AssetError::CreateDirectory {
            path: path.to_path_buf(),
            source,
        }
    })
}

pub struct BuildReport {
    pub pages_written: usize,
    pub assets_copied: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontMatter {
    title: String,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(default)]
    pub description: String,
}

fn default_template() -> String {
    "page.html".to_owned()
}

fn extract_front_matter(document: &str) -> Result<(&str, &str), FrontMatterError> {
    let document = document.strip_prefix('\u{feff}').unwrap_or(document);

    let remainder = document
        .strip_prefix("---\n")
        .ok_or(FrontMatterError::MissingOpeningDelimeter)?;

    let mut offset = 0;

    for line in remainder.split_inclusive('\n') {
        let line_without_ending = line.trim_end_matches(['\n']);

        if line_without_ending == "---" {
            let yaml = &remainder[..offset];
            let markdown = &remainder[offset + line.len()..];

            return Ok((yaml, markdown));
        }

        offset += line.len();
    }

    Err(FrontMatterError::MissingClosingDelimeter)
}

fn parse_document(document: &str) -> Result<(FrontMatter, &str), FrontMatterError> {
    let (yaml, markdown) = extract_front_matter(document)?;
    let front_matter = serde_yaml::from_str(yaml)?;

    Ok((front_matter, markdown))
}

struct Page {
    source_path: PathBuf,
    output_path: PathBuf,
    metadata: FrontMatter,
    html: String,
}
