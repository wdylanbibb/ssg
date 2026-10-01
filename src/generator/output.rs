use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use tempfile::{Builder, TempDir};

use super::error::{BuildError, ContentError, StagingError};

pub(super) struct OutputPaths {
    pub(super) root: PathBuf,
    pub(super) assets: PathBuf,
}

pub(super) struct StagedOutput {
    destination: PathBuf,
    staging: TempDir,
}

impl StagedOutput {
    pub(super) fn new(output: &Path) -> Result<Self, StagingError> {
        let parent = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        let file_name = output
            .file_name()
            .ok_or_else(|| StagingError::CreateStagingDirectory {
                path: output.to_path_buf(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "output path has no filename",
                ),
            })?;

        std::fs::create_dir_all(parent).map_err(|source| {
            StagingError::CreateStagingDirectory { path: parent.to_path_buf(), source }
        })?;

        let mut prefix = OsString::from(".");
        prefix.push(file_name);
        prefix.push(".build-");

        let staging = Builder::new()
            .prefix(&prefix)
            .tempdir_in(parent)
            .map_err(|source| StagingError::CreateStagingDirectory { path: parent.to_path_buf(), source })?;

        Ok(Self {
            destination: output.to_path_buf(),
            staging,
        })
    }

    pub(super) fn path(&self) -> &Path {
        self.staging.path()
    }

    pub(super) fn publish(self) -> Result<(), StagingError> {
        let backup = backup_path(self.staging.path());
        let had_previous_output = self.destination.try_exists()?;

        if had_previous_output {
            std::fs::rename(&self.destination, &backup)?;
        }

        let staging = self.staging.keep();

        if let Err(source) = std::fs::rename(&staging, &self.destination) {
            let _ = remove_path(&staging);

            if had_previous_output {
                let _ = std::fs::rename(&backup, &self.destination);
            }

            return Err(StagingError::Publish(source));
        }

        if had_previous_output {
            remove_path(&backup)?;
        }

        Ok(())
    }
}

fn backup_path(staging: &Path) -> PathBuf {
    let mut path = staging.as_os_str().to_os_string();
    path.push(".previous");
    PathBuf::from(path)
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;

    if metadata.file_type().is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

pub(super) fn prepare_output(output: &Path) -> Result<OutputPaths, BuildError> {
    let paths = OutputPaths {
        root: output.to_path_buf(),
        assets: output.join("assets"),
    };

    create_output_directory(&paths.root)?;
    create_output_directory(&paths.assets)?;

    Ok(paths)
}

pub(super) fn page_output_path(
    content_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, ContentError> {
    let relative = source_path.strip_prefix(content_root).map_err(|_| {
        ContentError::OutsideContentDirectory {
            path: source_path.to_path_buf(),
            content_root: content_root.to_path_buf(),
        }
    })?;

    if relative.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(ContentError::UnsupportedContentFile {
            path: source_path.to_path_buf(),
        });
    }

    let file_stem = relative
        .file_stem()
        .ok_or_else(|| ContentError::InvalidContentPath {
            path: source_path.to_path_buf(),
        })?;

    if file_stem == OsStr::new("index") {
        Ok(relative.with_extension("html"))
    } else {
        let mut output = relative.with_extension("");
        output.push("index.html");
        Ok(output)
    }
}

fn create_output_directory(path: &Path) -> Result<(), BuildError> {
    std::fs::create_dir_all(path).map_err(|source| BuildError::CreateOutputDirectory {
        path: path.to_path_buf(),
        source,
    })
}
