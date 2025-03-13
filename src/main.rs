use std::error::Error;

use camino::Utf8Path;
use log::{debug, error, info};

mod archive;
mod cli;
mod disprompt;
mod install;

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

fn select_archive(archive_groups: Vec<Vec<&Archive>>) -> Option<Archive> {
    let prompt = disprompt::DisambiguationPrompt::new(archive_groups);
    match prompt.run() {
        Ok(selected) => selected,
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn group_archives(archives: &Vec<Archive>) -> Vec<Vec<&Archive>> {
    let mut accum = Vec::new();
    let mut sub_accum: Vec<&Archive> = Vec::new();
    for arc in archives {
        if let Some(prev_arc) = sub_accum.last() {
            if prev_arc.name() != arc.name() {
                accum.push(sub_accum);
                sub_accum = Vec::new();
            }
        }
        sub_accum.push(arc);
    }

    if !sub_accum.is_empty() {
        accum.push(sub_accum);
    }

    accum
}

fn main_install(maybe_prefix: Option<String>) -> Result<(), Box<dyn Error>> {
    let archives = enumerate_archives()?;

    let matching_archives: Vec<_> = match maybe_prefix {
        Some(ref prefix) => archives
            .into_iter()
            .filter(|a| a.has_name(prefix.as_str()))
            .collect(),
        None => archives,
    };

    if matching_archives.is_empty() {
        if let Some(ref prefix) = maybe_prefix {
            error!("No archives found for '{}'.", prefix);
        } else {
            error!("No archives found.");
        }
        return Ok(());
    }

    let archive_groups = group_archives(&matching_archives);

    let selected_archive = select_archive(archive_groups);

    match selected_archive {
        Some(arc) => {
            install::install_archive(&arc)?;
        }
        None => {
            return Err("aborted without selecting an archive".into());
        }
    };

    Ok(())
}

fn main_latest(prefix: String) -> Result<(), Box<dyn Error>> {
    let archives = enumerate_archives()?;
    let mut matching_archives: Vec<_> = archives
        .iter()
        .filter(|a| a.has_name(prefix.as_str()))
        .collect();

    matching_archives.sort_by(|a, b| a.cmp_by_version(b));

    if let Some(arc) = matching_archives.last() {
        info!("{}", arc.base_name());
    } else {
        error!("Unable to determine latest archive name for: {}", prefix);
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
