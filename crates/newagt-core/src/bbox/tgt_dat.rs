//! Whitespace-delimited `tgt.dat` loader (legacy `TgtDB` semantics).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default fixture path bundled for tests (not used automatically at runtime).
pub const DEFAULT_TEST_TGT_DAT_SNIPPET: &str =
    "crates/newagt-core/tests/fixtures/tgt.dat.snippet";

#[derive(Debug, Clone, PartialEq)]
pub struct TgtDatEntry {
    pub tgt_type: String,
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
}

#[derive(Debug)]
pub enum TgtDatError {
    Io { path: PathBuf, source: std::io::Error },
    EmptyFile { path: PathBuf },
}

impl std::fmt::Display for TgtDatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::EmptyFile { path } => write!(f, "{}: empty tgt.dat", path.display()),
        }
    }
}

impl std::error::Error for TgtDatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::EmptyFile { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TgtDatDb {
    by_type: HashMap<String, TgtDatEntry>,
}

impl TgtDatDb {
    pub fn load(path: &Path) -> Result<Self, TgtDatError> {
        let text = fs::read_to_string(path).map_err(|source| TgtDatError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if text.trim().is_empty() {
            return Err(TgtDatError::EmptyFile {
                path: path.to_path_buf(),
            });
        }
        Ok(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let mut by_type = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 4 {
                continue;
            }
            let Ok(length_m) = cols[1].parse::<f64>() else {
                continue;
            };
            let Ok(width_m) = cols[2].parse::<f64>() else {
                continue;
            };
            let Ok(height_m) = cols[3].parse::<f64>() else {
                continue;
            };
            let entry = TgtDatEntry {
                tgt_type: cols[0].to_string(),
                length_m,
                width_m,
                height_m,
            };
            by_type.insert(cols[0].to_string(), entry);
        }
        Self { by_type }
    }

    pub fn get(&self, tgt_type: &str) -> Option<&TgtDatEntry> {
        self.by_type.get(tgt_type)
    }

    pub fn len(&self) -> usize {
        self.by_type.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_type.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_snippet_fixture() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tgt.dat.snippet");
        let db = TgtDatDb::load(&path).expect("load snippet");
        let m1 = db.get("M1").expect("M1");
        assert!((m1.length_m - 7.72).abs() < 1e-9);
        assert!((m1.width_m - 3.66).abs() < 1e-9);
        assert!((m1.height_m - 2.34).abs() < 1e-9);
    }
}
