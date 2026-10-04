use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use newagt_core::{ParseOptions, ParseProfile, VERSION};

#[derive(Parser)]
#[command(
    name = "newagt",
    version,
    about = "Read, validate, and convert imagery ground truth AGT containers"
)]
struct Cli {
    /// Parse profile: `agtj` (AGTJ-aligned, default) or `pdf1999` (PDF yacc strict).
    #[arg(long, value_name = "PROFILE", default_value = "agtj", global = true)]
    profile: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Summarize discovered AGT files and inferred version hints.
    Info {
        /// Game directory or basename path.
        path: PathBuf,
    },
    /// Human-readable dump (not yet implemented).
    Dump {
        path: PathBuf,
    },
    /// Cross-check ranges, pointers, and record sizes.
    Validate {
        path: PathBuf,
    },
    /// Export structured JSON (schema not yet defined).
    ToJson {
        path: PathBuf,
        /// Output file (stdout if omitted).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let profile = match ParseProfile::parse_name(&cli.profile) {
        Some(p) => p,
        None => {
            eprintln!(
                "newagt: unknown profile `{}` (expected `agtj` or `pdf1999`)",
                cli.profile
            );
            return ExitCode::from(2);
        }
    };
    let _parse_opts = ParseOptions { profile };

    match cli.command {
        Commands::Info { path } => stub("info", profile, &path),
        Commands::Dump { path } => stub("dump", profile, &path),
        Commands::Validate { path } => stub("validate", profile, &path),
        Commands::ToJson { path, output } => {
            let target = output
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<stdout>".to_string());
            eprintln!(
                "newagt to-json: not yet implemented (core {}, profile: {}, path: {}, output: {})",
                VERSION,
                profile,
                path.display(),
                target
            );
            ExitCode::from(2)
        }
    }
}

fn stub(subcommand: &str, profile: ParseProfile, path: &Path) -> ExitCode {
    eprintln!(
        "newagt {subcommand}: not yet implemented (core {}, profile: {}, path: {})",
        VERSION,
        profile,
        path.display()
    );
    ExitCode::from(2)
}
