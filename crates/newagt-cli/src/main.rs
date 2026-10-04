use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use newagt_core::{
    format_report_json, format_report_text, validate, ParseOptions, ParseProfile, VERSION,
};

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

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ReportFormat {
    Text,
    Json,
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
        /// Report format: `text` (default) or `json`.
        #[arg(long, value_enum, default_value = "text")]
        format: ReportFormat,
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
    let parse_opts = ParseOptions { profile };

    match cli.command {
        Commands::Info { path } => stub("info", profile, &path),
        Commands::Dump { path } => stub("dump", profile, &path),
        Commands::Validate { path, format } => run_validate(&path, parse_opts, format),
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

fn run_validate(path: &Path, options: ParseOptions, format: ReportFormat) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("newagt validate: {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };

    let report = validate(&source, options);
    let output = match format {
        ReportFormat::Text => format_report_text(&report),
        ReportFormat::Json => format_report_json(&report),
    };

    if let Err(e) = write_report_stdout(&output) {
        eprintln!("newagt validate: write stdout: {e}");
        return ExitCode::from(1);
    }

    if report.loadable {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn write_report_stdout(text: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
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
