use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use camino::{ReadDirUtf8, Utf8Path, Utf8PathBuf};
use log::debug;
use regex::Regex;

enum ArchiveFormat {
    Zip,
    TarBz2,
    TarGz,
    TarXz,
}

impl Display for ArchiveFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::TarBz2 => "bz2(tar)",
            ArchiveFormat::TarGz => "gz(tar)",
            ArchiveFormat::TarXz => "xz(tar)",
        })?;
        Ok(())
    }
}

struct Archive {
    path: Utf8PathBuf,
    extracted_path: Option<Utf8PathBuf>,
    base_name: String,
    format: ArchiveFormat,
    version: String,
    name: String,
}

impl Display for Archive {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.path.as_str())?;
        f.write_str(" [base_name=")?;
        f.write_str(self.base_name.as_str())?;
        f.write_str(", name=")?;
        f.write_str(self.name.as_str())?;
        f.write_str(", version=")?;
        f.write_str(self.version.as_str())?;
        f.write_str(", fmt=")?;
        f.write_str(self.format.to_string().as_str())?;
        f.write_str(", extracted=")?;
        f.write_fmt(format_args!("{}", self.extracted_path.is_some()))?;
        f.write_str("]")?;
        Ok(())
    }
}

fn match_archive_ext(file_name: &str) -> Option<(String, ArchiveFormat)> {
    let archive_ext_re =
        Regex::new(r"[.](tar[.](bz2|gz|xz)|zip)$").expect("regex compilation failure.");

    let caps = archive_ext_re.captures(file_name)?;
    let ext = caps.get(1)?;
    let fmt = match ext.as_str() {
        "tar.bz2" => Some(ArchiveFormat::TarBz2),
        "tar.gz" => Some(ArchiveFormat::TarGz),
        "tar.xz" => Some(ArchiveFormat::TarXz),
        "zip" => Some(ArchiveFormat::Zip),
        _ => None,
    };

    let ext_offset = caps.get(0)?.start();
    let base_name = file_name[0..ext_offset].to_string();
    Some((base_name, fmt?))
}

fn match_version(file_name: &str) -> Option<String> {
    let version_re =
        Regex::new(r"[-_](v[0-9]+([.][0-9]+){0,2}|[0-9]{4}[-]?[0-9]{2}[-]?[0-9]{2})[.]")
            .expect("rege compilation failure.");
    let caps = version_re.captures(file_name)?;
    Some(caps.get(1)?.as_str().to_string())
}

fn match_name_before_version(file_name: &str, version: &str) -> Option<String> {
    let offset = file_name.find(version)?;
    match offset {
        0..1 => None,
        off => Some(file_name[0..off - 1].to_string()),
    }
}

fn enumerate_archives() -> Result<Vec<Archive>, Box<dyn Error>> {
    let dir = Utf8Path::new(".").read_dir_utf8()?;
    let mut accum: Vec<Archive> = Vec::new();
    for entry in dir.filter_map(|e| e.ok()) {
        if let Ok(t) = entry.file_type() {
            if !t.is_file() {
                continue;
            }

            let (base_name, archive_format) = match match_archive_ext(entry.file_name()) {
                Some(pair) => pair,
                None => {
                    debug!("Ignoring file due to extension: {}", entry.file_name());
                    continue;
                }
            };

            let archive_version = match match_version(entry.file_name()) {
                Some(ver) => ver,
                None => {
                    debug!(
                        "Ignoring file due to missing version: {}",
                        entry.file_name()
                    );
                    continue;
                }
            };

            let name = match match_name_before_version(entry.file_name(), &archive_version) {
                Some(name) => name,
                None => {
                    debug!(
                        "Ignoring file due to missing name before version: {}",
                        entry.file_name()
                    );
                    continue;
                }
            };

            let extracted_path = {
                let expected_path = Utf8Path::new(base_name.as_str());
                if expected_path.exists() && expected_path.is_dir() {
                    Some(expected_path.to_path_buf())
                } else {
                    None
                }
            };

            accum.push(Archive {
                path: entry.into_path(),
                extracted_path: extracted_path,
                base_name: base_name,
                format: archive_format,
                version: archive_version,
                name: name,
            });
        }
    }

    Ok(accum)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let archives = enumerate_archives()?;

    for archive in archives {
        println!("{}", archive);
    }

    Ok(())
}
