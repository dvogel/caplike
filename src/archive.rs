use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use camino::{Utf8Path, Utf8PathBuf};
use derive_more::{Display, Error};
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub enum ArchiveFormat {
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

#[derive(Debug, PartialEq)]
pub struct Archive {
    // path: Utf8PathBuf,
    extracted_path: Option<Utf8PathBuf>,
    base_name: String,
    format: ArchiveFormat,
    version: String,
    name: String,
}

impl Display for Archive {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        // f.write_str(self.path.as_str())?;
        f.write_str("Archive")?;
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

#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum ArchiveErrorCode {
    #[display("Ignoring file due to extension.")]
    BadFileExtension,

    #[display("Ignoring file due to missing version.")]
    MissingVersion,

    #[display("Ignoring file due to missing name before version.")]
    MissingName,
}

#[derive(Debug, Display, Error, PartialEq, Eq)]
#[display("Bad archive name: {}: {}", file_name, error_code)]
pub struct ArchiveError {
    file_name: String,
    error_code: ArchiveErrorCode,
}

impl ArchiveError {
    pub fn new(file_name: &str, error_code: ArchiveErrorCode) -> Self {
        ArchiveError {
            file_name: file_name.to_string(),
            error_code,
        }
    }
}

impl Archive {
    pub fn has_name(&self, proposed: &str) -> bool {
        self.name == proposed
    }

    pub fn cmp_by_version(&self, other: &Self) -> Ordering {
        human_sort::compare(&self.version, &other.version)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn base_name(&self) -> &str {
        self.base_name.as_str()
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
            Regex::new(r"\b(v[0-9]+([.][0-9]+){0,2}|[0-9]{4}[-]?[0-9]{2}[-]?[0-9]{2})[.]")
                .expect("regex compilation failure.");
        let caps = version_re.captures(file_name)?;
        Some(caps.get(1)?.as_str().to_string())
    }

    fn match_name_before_version(file_name: &str, version: &str) -> Option<String> {
        let offset = file_name.find(version)?;
        match offset {
            0 | 1 => None,
            off => Some(file_name[0..off - 1].to_string()),
        }
    }

    pub fn from_file_name(file_name: &str) -> Result<Archive, ArchiveError> {
        let (base_name, archive_format) = Self::match_archive_ext(file_name).ok_or(
            ArchiveError::new(file_name, ArchiveErrorCode::BadFileExtension),
        )?;

        let archive_version = Self::match_version(file_name).ok_or(ArchiveError::new(
            file_name,
            ArchiveErrorCode::MissingVersion,
        ))?;

        let name = Self::match_name_before_version(file_name, &archive_version)
            .ok_or(ArchiveError::new(file_name, ArchiveErrorCode::MissingName))?;

        let extracted_path = {
            let expected_path = Utf8Path::new(base_name.as_str());
            if expected_path.exists() && expected_path.is_dir() {
                Some(expected_path.to_path_buf())
            } else {
                None
            }
        };

        Ok(Archive {
            // path: entry.into_path(),
            extracted_path,
            base_name,
            format: archive_format,
            version: archive_version,
            name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Archive, ArchiveError, ArchiveErrorCode, ArchiveFormat};
    use std::error::Error;

    #[test]
    fn test_basic_success() -> Result<(), Box<dyn Error>> {
        let arc = Archive::from_file_name("abc-123-2000-01-01.tar.gz")?;
        assert_eq!(arc.name, "abc-123");
        assert_eq!(arc.version, "2000-01-01");
        assert_eq!(arc.format, ArchiveFormat::TarGz);
        Ok(())
    }

    #[test]
    fn test_bad_extension() {
        let file_name = "abc-123-2000-01-01.7z";
        let arc_res = Archive::from_file_name(file_name);
        assert_eq!(
            arc_res,
            Err(ArchiveError::new(
                file_name,
                ArchiveErrorCode::BadFileExtension
            ))
        );
    }

    #[test]
    fn test_missing_version() {
        let file_name = "abc-123.zip";
        let arc_res = Archive::from_file_name(file_name);
        assert_eq!(
            arc_res,
            Err(ArchiveError::new(
                file_name,
                ArchiveErrorCode::MissingVersion
            ))
        );
    }

    #[test]
    fn test_missing_name() {
        let file_name = "2000-01-01.zip";
        let arc_res = Archive::from_file_name(file_name);
        assert_eq!(
            arc_res,
            Err(ArchiveError::new(file_name, ArchiveErrorCode::MissingName))
        );
    }
}
