use std::{
    collections::{BTreeMap, btree_map::Entry},
    path::{Path, PathBuf},
};

use crate::generator::{error::PlanError, source::SourcePaths};
use walkdir::WalkDir;

pub(super) struct PlannedPage {
    pub source: PathBuf,
    pub destination: PathBuf,
}

pub(super) struct PlannedAsset {
    pub source: PathBuf,
    pub destination: PathBuf,
}

enum Destination {
    File { source: PathBuf },
    Directory,
}

impl Destination {
    fn source_path(&self) -> Option<&Path> {
        match self {
            Self::File { source } => Some(source),
            Self::Directory => None,
        }
    }
}

struct DestinationRegistry {
    entries: BTreeMap<PathBuf, Destination>,
}

impl DestinationRegistry {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl DestinationRegistry {
    fn reserve_file(&mut self, destination: &Path, source: &Path) -> Result<(), PlanError> {
        if let Some(parent) = destination.parent() {
            self.reserve_directory(parent, source)?;
        }

        match self.entries.entry(destination.to_path_buf()) {
            Entry::Vacant(entry) => {
                entry.insert(Destination::File {
                    source: source.to_path_buf(),
                });
                Ok(())
            }
            Entry::Occupied(entry) => Err(PlanError::Collision {
                destination: destination.to_path_buf(),
                existing_source: entry.get().source_path().map(Path::to_path_buf),
                incoming_source: source.to_path_buf(),
            }),
        }
    }

    fn reserve_directory(&mut self, destination: &Path, source: &Path) -> Result<(), PlanError> {
        if let Some(parent) = destination.parent()
            && parent != destination
        {
            self.reserve_directory(parent, source)?;
        }

        match self.entries.entry(destination.to_path_buf()) {
            Entry::Vacant(entry) => {
                entry.insert(Destination::Directory);
                Ok(())
            }
            Entry::Occupied(entry) => match entry.get() {
                Destination::Directory => Ok(()),
                Destination::File {
                    source: existing_source,
                } => Err(PlanError::Collision {
                    destination: destination.to_path_buf(),
                    existing_source: Some(existing_source.clone()),
                    incoming_source: source.to_path_buf(),
                }),
            },
        }
    }
}

pub(super) struct BuildPlan {
    pub pages: Vec<PlannedPage>,
    pub assets: Vec<PlannedAsset>,
}

impl BuildPlan {
    pub(super) fn discover(source_paths: &SourcePaths) -> Result<BuildPlan, PlanError> {
        let mut registry = DestinationRegistry::new();
        let mut pages = Vec::new();
        let mut assets = Vec::new();

        for entry in WalkDir::new(&source_paths.content).follow_links(false) {
            let entry = entry?;
            let source = entry.path();

            if !entry.file_type().is_file()
                || source.extension().and_then(|extension| extension.to_str()) != Some("md")
            {
                continue;
            }

            let destination = super::output::page_output_path(&source_paths.content, source)?;
            registry.reserve_file(&destination, source)?;
            pages.push(PlannedPage {
                source: source.to_path_buf(),
                destination,
            });
        }

        for entry in WalkDir::new(&source_paths.assets).follow_links(false) {
            let entry = entry?;
            let source = entry.path();
            let relative = source.strip_prefix(&source_paths.assets).map_err(|_| {
                PlanError::OutsideAssetsDirectory {
                    path: source.to_path_buf(),
                    assets_root: source_paths.assets.clone(),
                }
            })?;

            if relative.as_os_str().is_empty() {
                continue;
            }

            let destination = Path::new("assets").join(relative);
            if entry.file_type().is_dir() {
                registry.reserve_directory(&destination, source)?;
            } else if entry.file_type().is_file() {
                registry.reserve_file(&destination, source)?;
                assets.push(PlannedAsset {
                    source: source.to_path_buf(),
                    destination,
                });
            } else {
                return Err(PlanError::UnsupportedAssetType {
                    path: source.to_path_buf(),
                });
            }
        }

        Ok(BuildPlan { pages, assets })
    }
}
