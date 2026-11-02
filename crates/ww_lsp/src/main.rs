//! `ww_lsp`: the WireWeaver language server (`ww_lsp serve`, what an editor starts) and a small CLI on the same
//! analysis, for tests and scripts (`ww_lsp hints FILE`, `ww_lsp hover FILE:LINE:COL`).

mod complete;
mod output;
mod server;
mod usage;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::ArgValueCompleter;
use std::path::PathBuf;
use std::process::ExitCode;

/// `ww_lsp 0.1.0 (a1b2c3d, commit 2 Nov 2026 09:10 UTC)`.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_SHA"),
    ", commit ",
    env!("BUILD_TIME"),
    ")"
);

#[derive(Parser)]
#[command(
    name = "ww_lsp",
    version = VERSION,
    about = "A WireWeaver language server beside rust-analyzer: exact_units types as inlay hints and hover",
    after_help = "Examples:\n  ww_lsp serve                      the server, on stdin/stdout (what an editor starts)\n  ww_lsp hints src/lib.rs           every hint of a file, one per line\n  ww_lsp hover src/lib.rs:24:30     the hover at line 24, column 30 (1-based)\n\nShell completion: source <(COMPLETE=bash ww_lsp)   (zsh, fish: the same with their name)"
)]
struct Cli {
    /// Colour in the output: auto (a terminal without NO_COLOR), always or never
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto, global = true)]
    color: ColorWhen,

    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, ValueEnum)]
enum ColorWhen {
    /// Colour when stdout is a terminal and `NO_COLOR` is not set
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Command {
    /// Run the language server on stdin and stdout: inlay hints and hover for the exact_units types of #[ww_trait] items
    Serve {
        /// Accepted for clients that pass it (VS Code's); stdio is the only transport
        #[arg(long)]
        stdio: bool,
    },
    /// Print the inlay hints of Rust files: `FILE:LINE:COL  HINT  (ITEM)`, positions 1-based, at the end of each type
    Hints {
        /// Rust sources with #[ww_trait] traits
        #[arg(required = true, add = ArgValueCompleter::new(complete::rust_files))]
        files: Vec<PathBuf>,
    },
    /// Print the hover of the type at a position, Markdown, as an editor shows it
    Hover {
        /// FILE:LINE:COL, 1-based, the column in characters
        #[arg(add = ArgValueCompleter::new(complete::rust_files))]
        at: String,
    },
    /// How often each subcommand ran on this PC (from the local usage record)
    Usage,
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Command::Serve { .. } => "serve",
            Command::Hints { .. } => "hints",
            Command::Hover { .. } => "hover",
            Command::Usage => "usage",
        }
    }
}

fn main() -> ExitCode {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();
    match cli.color {
        ColorWhen::Auto => {}
        ColorWhen::Always => anstream::ColorChoice::Always.write_global(),
        ColorWhen::Never => anstream::ColorChoice::Never.write_global(),
    }
    usage::record(cli.command.name());
    let result = match cli.command {
        Command::Serve { stdio: _ } => server::run(),
        Command::Hints { files } => output::hints(&files),
        Command::Hover { at } => output::hover(&at),
        Command::Usage => usage::show(),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            output::error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Errors of a subcommand, shown as `error: ...`.
type Error = Box<dyn std::error::Error + Send + Sync>;
