use std::{fs, path::Path};

use super::{error::AssetError, plan::PlannedAsset};

pub(super) fn copy_assets(assets: &[PlannedAsset], output: &Path) -> Result<usize, AssetError> {
    let mut files_copied = 0;

    for asset in assets {
        let destination = output.join(&asset.destination);
        if let Some(parent) = destination.parent() {
            create_asset_directory(parent)?;
        }

        fs::copy(&asset.source, &destination).map_err(|source| AssetError::Copy {
            source_path: asset.source.clone(),
            destination,
            source,
        })?;

        files_copied += 1;
    }

    Ok(files_copied)
}

fn create_asset_directory(path: &Path) -> Result<(), AssetError> {
    fs::create_dir_all(path).map_err(|source| AssetError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}
