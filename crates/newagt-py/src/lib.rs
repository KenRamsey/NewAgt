//! PyO3 bindings for `newagt-core` (parse, validate, frames, bboxes).

use std::fs;
use std::path::{Path, PathBuf};

use newagt_core::{
    build_frame_index, export_parse_result, parse_with_options, resolve_bboxes, summarize_document,
    validate, BboxMethod, BBoxOptions, Document, FrameIndexOptions, JsonExportOptions,
    ParseOptions, ParseProfile, ParseResult,
};
use pyo3::exceptions::{PyFileNotFoundError, PyIOError, PyValueError};
use pyo3::prelude::*;
use pythonize::pythonize;

fn parse_profile_name(name: &str) -> PyResult<ParseProfile> {
    ParseProfile::parse_name(name).ok_or_else(|| {
        PyValueError::new_err(format!(
            "unknown profile `{name}` (expected `agtj` or `pdf1999`)"
        ))
    })
}

fn parse_bbox_method(name: &str) -> PyResult<BboxMethod> {
    match name {
        "score" => Ok(BboxMethod::Score),
        "tgtdb" => Ok(BboxMethod::Tgtdb),
        _ => Err(PyValueError::new_err(format!(
            "unknown bbox method `{name}` (expected `score` or `tgtdb`)"
        ))),
    }
}

/// Read AGT source from a filesystem path or treat `path_or_str` as literal AGT text.
fn read_source(path_or_str: &str) -> PyResult<(String, Option<String>)> {
    let path = Path::new(path_or_str);
    if path.is_file() {
        let text = fs::read_to_string(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        Ok((text, Some(path_or_str.to_string())))
    } else {
        Ok((path_or_str.to_string(), None))
    }
}

fn read_path(path: &str) -> PyResult<(String, String)> {
    let p = Path::new(path);
    if !p.is_file() {
        return Err(PyFileNotFoundError::new_err(format!("not a file: {path}")));
    }
    let text = fs::read_to_string(p).map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok((text, path.to_string()))
}

fn json_value_to_py<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> PyResult<Bound<'py, PyAny>> {
    Ok(pythonize(py, value).map_err(|e| PyValueError::new_err(e.to_string()))?)
}

fn parse_inner(source: &str, profile: ParseProfile) -> PyResult<ParseResult> {
    parse_with_options(source, ParseOptions { profile }).map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Handle to a parsed AGT document (keeps the Rust AST alive across Python calls).
#[pyclass(name = "ParsedDocument")]
pub struct ParsedDocument {
    inner: Document,
    source_path: Option<String>,
    profile: ParseProfile,
}

#[pymethods]
impl ParsedDocument {
    #[getter]
    fn profile(&self) -> &'static str {
        self.profile.as_str()
    }

    #[getter]
    fn path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// JSON-serializable summary counts and metadata.
    fn summary(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let s = summarize_document(&self.inner);
        let value = serde_json::json!({
            "profile": self.profile.as_str(),
            "path": self.source_path,
            "sections": {
                "prj_sect": s.prj_sect,
                "sen_sect": s.sen_sect,
                "tgt_sect": s.tgt_sect,
            },
            "updates": {
                "sen_upd": s.sen_upd,
                "tgt_upd": s.tgt_upd,
            },
            "targets": s.tgt,
        });
        json_value_to_py(py, &value).map(|b| b.unbind())
    }

    fn __repr__(&self) -> String {
        let s = summarize_document(&self.inner);
        format!(
            "ParsedDocument(profile={}, tgt={}, path={:?})",
            self.profile.as_str(),
            s.tgt,
            self.source_path
        )
    }
}

/// Parse AGT from a file path or inline source string.
#[pyfunction]
#[pyo3(signature = (path_or_str, profile="agtj"))]
fn parse(path_or_str: &str, profile: &str) -> PyResult<ParsedDocument> {
    let profile = parse_profile_name(profile)?;
    let (source, path) = read_source(path_or_str)?;
    let result = parse_inner(&source, profile)?;
    Ok(ParsedDocument {
        inner: result.document,
        source_path: path,
        profile,
    })
}

/// Validate an AGT file; returns a JSON-serializable report dict.
#[pyfunction]
#[pyo3(name = "validate", signature = (path, profile="agtj"))]
fn validate_py(path: &str, profile: &str, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let profile = parse_profile_name(profile)?;
    let (source, _) = read_path(path)?;
    let report = validate(&source, ParseOptions { profile });
    let json = newagt_core::format_report_json(&report);
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_value_to_py(py, &value).map(|b| b.unbind())
}

/// Build trainer-facing frame records from an AGT file.
#[pyfunction]
#[pyo3(signature = (path, heuristic=false))]
fn frames(path: &str, heuristic: bool, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let (source, _) = read_path(path)?;
    let result = parse_inner(&source, ParseProfile::default())?;
    let options = FrameIndexOptions {
        use_agtj_heuristics: heuristic,
    };
    let index = build_frame_index(&result.document, options);
    let value = serde_json::to_value(&index.frames)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_value_to_py(py, &value).map(|b| b.unbind())
}

/// Resolve bounding boxes for targets across frames.
#[pyfunction]
#[pyo3(signature = (path, image_width, image_height, fov_h, fov_v, tgt_dat=None, method="score", heuristic=false))]
fn bboxes(
    path: &str,
    image_width: u32,
    image_height: u32,
    fov_h: f64,
    fov_v: f64,
    tgt_dat: Option<&str>,
    method: &str,
    heuristic: bool,
    py: Python<'_>,
) -> PyResult<Py<PyAny>> {
    let (source, _) = read_path(path)?;
    let result = parse_inner(&source, ParseProfile::default())?;
    let method = parse_bbox_method(method)?;
    let tgt_dat_path = tgt_dat.map(PathBuf::from);
    let options = BBoxOptions {
        tgt_dat_path,
        image_width: Some(image_width),
        image_height: Some(image_height),
        fov_h_deg: Some(fov_h),
        fov_v_deg: Some(fov_v),
        method,
        frame_index: FrameIndexOptions {
            use_agtj_heuristics: heuristic,
        },
        ..Default::default()
    };
    let index = resolve_bboxes(&result.document, &options).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let value = serde_json::to_value(&index.records)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_value_to_py(py, &value).map(|b| b.unbind())
}

/// Export `newagt.schema.v1` JSON for an AGT file.
#[pyfunction]
#[pyo3(signature = (path, profile="agtj", spans=false))]
fn to_json(path: &str, profile: &str, spans: bool, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let profile = parse_profile_name(profile)?;
    let (source, _file_path) = read_path(path)?;
    let result = parse_inner(&source, profile)?;
    let json_str = export_parse_result(
        &result,
        profile,
        JsonExportOptions {
            include_spans: spans,
        },
    );
    let value: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| PyValueError::new_err(e.to_string()))?;
    json_value_to_py(py, &value).map(|b| b.unbind())
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", newagt_core::VERSION)?;
    m.add_class::<ParsedDocument>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(validate_py, m)?)?;
    m.add_function(wrap_pyfunction!(frames, m)?)?;
    m.add_function(wrap_pyfunction!(bboxes, m)?)?;
    m.add_function(wrap_pyfunction!(to_json, m)?)?;
    Ok(())
}
