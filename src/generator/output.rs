use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use super::error::{BuildError, ContentError};

pub(super) struct OutputPaths {
    pub(super) root: PathBuf,
    pub(super) assets: PathBuf,
}

fn create_output_directory(path: &Path) -> Result<(), BuildError> {
    std::fs::create_dir_all(path).map_err(|source| BuildError::CreateOutputDirectory {
        path: path.to_path_buf(),
        source,
    })
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
