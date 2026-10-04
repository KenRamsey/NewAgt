use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use newagt_core::{
    build_frame_index, dump_document, export_parse_result, format_frame_line, format_info,
    format_info_batch_line, format_report_json, format_report_text, resolve_bboxes,
    scan_classic_dataset_pairs, sections_present, summarize_document, validate, BboxMethod,
    BBoxOptions, DatasetPairRow, FrameIndexOptions, JsonExportOptions, MissingSideFlags,
    ParseOptions, ParseProfile,
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

    /// Optional authority frame count for indexing and validation (ARF not read yet).
    #[arg(long, value_name = "N", global = true)]
    frame_count: Option<u32>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ReportFormat {
    Text,
    Json,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MissingSideCli {
    /// Report `.agt` files with no matching `.arf` (same basename).
    Agt,
    /// Report `.arf` files with no matching `.agt`.
    Arf,
    /// Report both orphan kinds.
    Both,
}

impl From<MissingSideCli> for MissingSideFlags {
    fn from(v: MissingSideCli) -> Self {
        match v {
            MissingSideCli::Agt => MissingSideFlags {
                report_missing_agt: true,
                report_missing_arf: false,
            },
            MissingSideCli::Arf => MissingSideFlags {
                report_missing_agt: false,
                report_missing_arf: true,
            },
            MissingSideCli::Both => MissingSideFlags {
                report_missing_agt: true,
                report_missing_arf: true,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BboxMethodCli {
    Score,
    Tgtdb,
}

impl From<BboxMethodCli> for BboxMethod {
    fn from(v: BboxMethodCli) -> Self {
        match v {
            BboxMethodCli::Score => BboxMethod::Score,
            BboxMethodCli::Tgtdb => BboxMethod::Tgtdb,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Summarize AGT file path, profile, and section counts (or one line per `.agt` under a directory).
    Info {
        path: PathBuf,
        /// When PATH is a directory, include `.agt` files in subdirectories (default: true).
        #[arg(long, default_value_t = true)]
        recursive: bool,
        /// When PATH is a directory, process at most N `.agt` files (after sorting paths).
        #[arg(long, value_name = "N")]
        limit: Option<usize>,
    },
    /// List trainer frame index (one summary line per frame).
    Frames {
        path: PathBuf,
        /// Pair updates using AGTJ-style Keyword/Comment frame hints when possible.
        #[arg(long)]
        heuristic: bool,
    },
    /// Human-readable indented dump of parsed structure.
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
    /// Resolve trainer bounding boxes per target (PixBox or computed).
    Bboxes {
        path: PathBuf,
        /// Target dimension database (`tgt.dat`).
        #[arg(long, value_name = "PATH")]
        tgt_dat: Option<PathBuf>,
        #[arg(long, value_name = "PX")]
        image_width: Option<u32>,
        #[arg(long, value_name = "PX")]
        image_height: Option<u32>,
        /// Horizontal FOV in degrees when not present in AGT.
        #[arg(long, value_name = "DEG")]
        fov_h: Option<f64>,
        /// Vertical FOV in degrees when not present in AGT.
        #[arg(long, value_name = "DEG")]
        fov_v: Option<f64>,
        /// Computed-box method: `score` (atan, normative) or `tgtdb` (linear legacy).
        #[arg(long, value_enum, default_value = "score")]
        method: BboxMethodCli,
        /// Recompute boxes even when `PixBox` is present on the target.
        #[arg(long)]
        ignore_pix_box: bool,
        /// Pair updates using AGTJ-style Keyword/Comment frame hints when possible.
        #[arg(long)]
        heuristic: bool,
    },
    /// Export structured JSON (`newagt.schema.v1`).
    ToJson {
        path: PathBuf,
        /// Output file (stdout if omitted).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include source line/column/byte spans in JSON nodes.
        #[arg(long)]
        spans: bool,
    },
    /// List ARF/AGT basename pairs in a classic per-sensor dataset layout.
    Pairs {
        /// Dataset root (one subdirectory per sensor, each with `arf/` and `agt/`).
        #[arg(long, value_name = "DIR")]
        dataset_root: PathBuf,
        /// Also report orphan files: `agt`, `arf`, or `both` sides.
        #[arg(long, value_enum, value_name = "SIDE")]
        missing: Option<MissingSideCli>,
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

    let frame_opts = |heuristic: bool| FrameIndexOptions {
        use_agtj_heuristics: heuristic,
        expected_frame_count: cli.frame_count,
    };

    match cli.command {
        Commands::Info {
            path,
            recursive,
            limit,
        } => run_info(&path, profile, parse_opts, recursive, limit, cli.frame_count),
        Commands::Frames { path, heuristic } => run_frames(&path, parse_opts, frame_opts(heuristic)),
        Commands::Dump { path } => run_dump(&path, parse_opts),
        Commands::Validate { path, format } => {
            run_validate(&path, parse_opts, format, cli.frame_count)
        }
        Commands::Bboxes {
            path,
            tgt_dat,
            image_width,
            image_height,
            fov_h,
            fov_v,
            method,
            ignore_pix_box,
            heuristic,
        } => run_bboxes(
            &path,
            parse_opts,
            BboxesRun {
                tgt_dat,
                image_width,
                image_height,
                fov_h,
                fov_v,
                method: method.into(),
                ignore_pix_box,
                frame_index: frame_opts(heuristic),
            },
        ),
        Commands::ToJson { path, output, spans } => run_to_json(&path, parse_opts, output, spans),
        Commands::Pairs {
            dataset_root,
            missing,
        } => run_pairs(
            &dataset_root,
            missing.map(Into::into).unwrap_or_default(),
        ),
    }
}

fn read_source(path: &Path) -> Result<String, ExitCode> {
    fs::read_to_string(path).map_err(|e| {
        eprintln!("newagt: {}: {e}", path.display());
        ExitCode::from(1)
    })
}

fn collect_agt_files(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), ExitCode> {
    let entries = fs::read_dir(dir).map_err(|e| {
        eprintln!("newagt info: {}: {e}", dir.display());
        ExitCode::from(1)
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            eprintln!("newagt info: {}: {e}", dir.display());
            ExitCode::from(1)
        })?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_agt_files(&path, true, out)?;
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("agt"))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn run_info(
    path: &Path,
    profile: ParseProfile,
    options: ParseOptions,
    recursive_flag: bool,
    limit: Option<usize>,
    authority_frame_count: Option<u32>,
) -> ExitCode {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("newagt info: {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };

    if meta.is_dir() {
        return run_info_directory(path, options, recursive_flag, limit, authority_frame_count);
    }

    if limit.is_some() {
        eprintln!("newagt info: --limit applies only when PATH is a directory");
        return ExitCode::from(2);
    }

    run_info_file(path, profile, options, authority_frame_count)
}

fn run_info_directory(
    dir: &Path,
    options: ParseOptions,
    recursive: bool,
    limit: Option<usize>,
    authority_frame_count: Option<u32>,
) -> ExitCode {
    let mut files = Vec::new();
    if let Err(code) = collect_agt_files(dir, recursive, &mut files) {
        return code;
    }
    files.sort();
    if let Some(n) = limit {
        files.truncate(n);
    }

    let mut text = String::new();
    let mut any_fail = false;
    for file in &files {
        let display = file.display().to_string();
        match fs::read_to_string(file) {
            Ok(source) => match newagt_core::parse_with_options(&source, options) {
                Ok(result) => {
                    let summary = summarize_document(&result.document);
                    let frames = build_frame_index(
                        &result.document,
                        FrameIndexOptions {
                            expected_frame_count: authority_frame_count,
                            ..FrameIndexOptions::DEFAULT
                        },
                    );
                    text.push_str(&format_info_batch_line(
                        &display,
                        Some(frames.len()),
                        &sections_present(&summary),
                        true,
                    ));
                }
                Err(_) => {
                    any_fail = true;
                    text.push_str(&format_info_batch_line(&display, None, "-", false));
                }
            },
            Err(_) => {
                any_fail = true;
                text.push_str(&format_info_batch_line(&display, None, "-", false));
            }
        }
    }

    if write_stdout(&text).is_err() {
        return ExitCode::from(1);
    }
    if any_fail {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_info_file(
    path: &Path,
    profile: ParseProfile,
    options: ParseOptions,
    authority_frame_count: Option<u32>,
) -> ExitCode {
    let source = match read_source(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match newagt_core::parse_with_options(&source, options) {
        Ok(result) => {
            let summary = summarize_document(&result.document);
            let frames = build_frame_index(
                &result.document,
                FrameIndexOptions {
                    expected_frame_count: authority_frame_count,
                    ..FrameIndexOptions::DEFAULT
                },
            );
            let text = format_info(
                &path.display().to_string(),
                profile.as_str(),
                &summary,
                &frames,
            );
            if write_stdout(&text).is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("newagt info: parse failed: {e}");
            ExitCode::from(1)
        }
    }
}

struct BboxesRun {
    tgt_dat: Option<PathBuf>,
    image_width: Option<u32>,
    image_height: Option<u32>,
    fov_h: Option<f64>,
    fov_v: Option<f64>,
    method: BboxMethod,
    ignore_pix_box: bool,
    frame_index: FrameIndexOptions,
}

fn run_bboxes(path: &Path, options: ParseOptions, run: BboxesRun) -> ExitCode {
    let source = match read_source(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match newagt_core::parse_with_options(&source, options) {
        Ok(result) => {
            let bbox_opts = BBoxOptions {
                tgt_dat_path: run.tgt_dat,
                image_width: run.image_width,
                image_height: run.image_height,
                fov_h_deg: run.fov_h,
                fov_v_deg: run.fov_v,
                default_fov: match (run.fov_h, run.fov_v) {
                    (Some(h), Some(v)) => Some(newagt_core::Fov {
                        horizontal: h,
                        vertical: v,
                    }),
                    _ => None,
                },
                method: run.method,
                ignore_pix_box: run.ignore_pix_box,
                frame_index: run.frame_index,
            };
            match resolve_bboxes(&result.document, &bbox_opts) {
                Ok(index) => {
                    let json = serde_json::to_string_pretty(&index).unwrap_or_else(|e| {
                        eprintln!("newagt bboxes: json encode: {e}");
                        String::new()
                    });
                    if json.is_empty() {
                        return ExitCode::from(1);
                    }
                    if write_stdout(&json).is_err() {
                        return ExitCode::from(1);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("newagt bboxes: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Err(e) => {
            eprintln!("newagt bboxes: parse failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_frames(path: &Path, options: ParseOptions, frame_index: FrameIndexOptions) -> ExitCode {
    let source = match read_source(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match newagt_core::parse_with_options(&source, options) {
        Ok(result) => {
            let index = build_frame_index(&result.document, frame_index);
            let mut text = String::new();
            if !index.warnings.is_empty() {
                for w in &index.warnings {
                    text.push_str("warning: ");
                    text.push_str(w);
                    text.push('\n');
                }
            }
            for frame in &index.frames {
                text.push_str(&format_frame_line(frame));
                text.push('\n');
            }
            if write_stdout(&text).is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("newagt frames: parse failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_dump(path: &Path, options: ParseOptions) -> ExitCode {
    let source = match read_source(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match newagt_core::parse_with_options(&source, options) {
        Ok(result) => {
            let text = dump_document(&result.document);
            if write_stdout(&text).is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("newagt dump: parse failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_to_json(
    path: &Path,
    options: ParseOptions,
    output: Option<PathBuf>,
    spans: bool,
) -> ExitCode {
    let source = match read_source(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match newagt_core::parse_with_options(&source, options) {
        Ok(result) => {
            let json = export_parse_result(
                &result,
                options.profile,
                JsonExportOptions {
                    include_spans: spans,
                },
            );
            if let Some(out_path) = output {
                if let Err(e) = fs::write(&out_path, &json) {
                    eprintln!("newagt to-json: {}: {e}", out_path.display());
                    return ExitCode::from(1);
                }
            } else if write_stdout(&json).is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("newagt to-json: parse failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_validate(
    path: &Path,
    options: ParseOptions,
    format: ReportFormat,
    authority_frame_count: Option<u32>,
) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("newagt validate: {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };

    let report = validate(&source, options, authority_frame_count);
    let output = match format {
        ReportFormat::Text => format_report_text(&report),
        ReportFormat::Json => format_report_json(&report),
    };

    if let Err(e) = write_stdout(&output) {
        eprintln!("newagt validate: write stdout: {e}");
        return ExitCode::from(1);
    }

    if report.loadable {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn format_pair_line(row: &DatasetPairRow) -> String {
    let arf = row
        .arf_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let agt = row
        .agt_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    format!("{}\t{}\t{}\t{}", row.sensor, arf, agt, row.stem)
}

fn run_pairs(dataset_root: &Path, missing: MissingSideFlags) -> ExitCode {
    let meta = match fs::metadata(dataset_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("newagt pairs: {}: {e}", dataset_root.display());
            return ExitCode::from(1);
        }
    };
    if !meta.is_dir() {
        eprintln!(
            "newagt pairs: {}: not a directory (expected classic dataset root)",
            dataset_root.display()
        );
        return ExitCode::from(2);
    }

    match scan_classic_dataset_pairs(dataset_root, missing) {
        Ok(rows) => {
            let mut text = String::new();
            for row in &rows {
                text.push_str(&format_pair_line(row));
                text.push('\n');
            }
            if write_stdout(&text).is_err() {
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("newagt pairs: {}: {e}", dataset_root.display());
            ExitCode::from(1)
        }
    }
}

fn write_stdout(text: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    Ok(())
}
