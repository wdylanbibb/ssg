use std::{collections::BTreeMap, path::{Path, PathBuf}};

use crate::generator::{error::PlanError, source::SourcePaths};

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

struct DestinationRegistry {
    entries: BTreeMap<PathBuf, Destination>,
}

pub(super) struct BuildPlan {
    pub pages: Vec<PlannedPage>,
    pub assets: Vec<PlannedAsset>,
}

impl BuildPlan {
    pub(super) fn discover(source_paths: &SourcePaths) -> Result<BuildPlan, PlanError> {
        todo!()
    }

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

    fn reserve_directory(&mut self, parent: &Path, source: &Path) -> Result<(), PlanError> {
        todo!()
    }
}
