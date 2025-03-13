use std::{error::Error, fs::File, path::Path, process};

use crate::archive::Archive;

pub fn install_archive(archive: &Archive) -> Result<(), Box<dyn Error>> {
    ensure_top_level_dir(archive, archive.base_name())?;

    if !Path::new(archive.base_name()).is_dir() {
        let _unzip_output = process::Command::new("unzip")
            .arg(archive.path())
            .output()?;
    }

    if let Some(extracted_path) = archive.extracted_path() {
        std::fs::remove_file(extracted_path.as_str())?;
    }
    std::os::unix::fs::symlink(archive.base_name(), archive.name()).map_err(|_| {
        format!(
            "Failed to create symlink '{}' to '{}'",
            archive.name(),
            archive.base_name()
        )
    })?;
    Ok(())
}

fn ensure_top_level_dir(archive: &Archive, expected_dir: &str) -> Result<(), Box<dyn Error>> {
    let fd = File::open(archive.path())?;
    let mut zip = zip::ZipArchive::new(fd)?;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        if !file.name().starts_with(expected_dir) {
            return Err(format!(
                "prefix mismatch for '{}' expecting '{}'",
                file.name(),
                expected_dir
            )
            .into());
        }
    }

    Ok(())
}
