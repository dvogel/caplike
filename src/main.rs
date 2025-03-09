use std::error::Error;

use camino::Utf8Path;
use log::debug;

mod archive;
mod cli;

use archive::Archive;
use cli::Subcommands;

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

fn main_install(maybe_prefix: Option<String>) -> Result<(), Box<dyn Error>> {
    Err("This command is not yet implemented.".into())
}

fn main_latest(prefix: String) -> Result<(), Box<dyn Error>> {
    let archives = enumerate_archives()?;
    let mut matching_archives: Vec<_> = archives
        .iter()
        .filter(|a| a.has_name(prefix.as_str()))
        .collect();

    matching_archives.sort_by(|a, b| a.cmp_by_version(b));

    if let Some(arc) = matching_archives.last() {
        println!("{}", arc.base_name());
    } else {
        eprintln!("Unable to determine latest archive name for: {}", prefix);
    }

    Ok(())
}

fn main_revert(maybe_prefix: Option<String>) -> Result<(), Box<dyn Error>> {
    Err("This command is not yet implemented.".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = cli::parse();

    match args.subcmd {
        Subcommands::Install { maybe_prefix } => main_install(maybe_prefix),
        Subcommands::Latest { prefix } => main_latest(prefix),
        Subcommands::Revert { maybe_prefix } => main_revert(maybe_prefix),
    }?;

    Ok(())
}
