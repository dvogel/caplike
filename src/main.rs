use std::error::Error;

use camino::Utf8Path;
use log::debug;

mod archive;

use archive::Archive;

fn enumerate_archives() -> Result<Vec<Archive>, Box<dyn Error>> {
    let dir = Utf8Path::new(".").read_dir_utf8()?;
    let mut accum: Vec<Archive> = Vec::new();
    for entry in dir.filter_map(|e| e.ok()) {
        if let Ok(t) = entry.file_type() {
            if !t.is_file() {
                continue;
            }

            match Archive::from_file_name(entry.file_name()) {
                Ok(archive) => accum.push(archive),
                Err(e) => debug!("{}", e),
            };
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
