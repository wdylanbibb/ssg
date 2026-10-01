use std::{fs, path::Path};

use walkdir::WalkDir;

use super::error::AssetError;

pub(super) fn copy_assets(assets_root: &Path, output_assets: &Path) -> Result<usize, AssetError> {
    create_asset_directory(output_assets)?;

    let mut files_copied = 0;

    for entry in WalkDir::new(assets_root).follow_links(false) {
        let entry = entry?;
        let source_path = entry.path();

        let relative_path = source_path.strip_prefix(assets_root).map_err(|_| {
            AssetError::OutsideAssetsDirectory {
                path: source_path.to_path_buf(),
                assets_root: assets_root.to_path_buf(),
            }
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

            fs::copy(source_path, &destination).map_err(|source| AssetError::Copy {
                source_path: source_path.to_path_buf(),
                destination,
                source,
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
    fs::create_dir_all(path).map_err(|source| AssetError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}
