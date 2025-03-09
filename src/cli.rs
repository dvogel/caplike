use clap::{Parser, Subcommand};

// caplike latest <prefix>
// caplike install [prefix]
// caplike revert [prefix]

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CmdArgs {
    #[command(subcommand)]
    pub subcmd: Subcommands,
}

#[derive(Clone, Subcommand)]
pub enum Subcommands {
    Latest { prefix: String },
    Install { maybe_prefix: Option<String> },
    Revert { maybe_prefix: Option<String> },
}

/// Simple alias for
pub fn parse() -> CmdArgs {
    CmdArgs::parse()
}
