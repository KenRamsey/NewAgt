//! Classic Ken-style dataset layout: one directory per sensor with paired `arf/` and `agt/` trees.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Which orphan sides to include when scanning a classic dataset layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissingSideFlags {
    pub report_missing_agt: bool,
    pub report_missing_arf: bool,
}

/// One row from a classic layout scan (paths are runtime values, not repo fixtures).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetPairRow {
    pub sensor: String,
    pub arf_path: Option<PathBuf>,
    pub agt_path: Option<PathBuf>,
    pub stem: String,
}

/// Walk `$DATASET_ROOT/<sensor>/{arf,agt}/` and collect basename pairs (`.arf` ↔ `.agt`).
pub fn scan_classic_dataset_pairs(
    dataset_root: &Path,
    missing: MissingSideFlags,
) -> io::Result<Vec<DatasetPairRow>> {
    let mut rows = Vec::new();

    let sensor_entries = fs::read_dir(dataset_root)?;
    for sensor_entry in sensor_entries {
        let sensor_entry = sensor_entry?;
        let sensor_path = sensor_entry.path();
        if !sensor_path.is_dir() {
            continue;
        }
        let sensor_name = sensor_entry
            .file_name()
            .to_string_lossy()
            .into_owned();

        let arf_dir = sensor_path.join("arf");
        let agt_dir = sensor_path.join("agt");

        let arf_by_stem = list_files_by_stem(&arf_dir, "arf")?;
        let agt_by_stem = list_files_by_stem(&agt_dir, "agt")?;

        let mut stems: BTreeSet<String> = BTreeSet::new();
        stems.extend(arf_by_stem.keys().cloned());
        stems.extend(agt_by_stem.keys().cloned());

        for stem in stems {
            let arf_path = arf_by_stem.get(&stem).cloned();
            let agt_path = agt_by_stem.get(&stem).cloned();

            match (arf_path.as_ref(), agt_path.as_ref()) {
                (Some(_), Some(_)) => rows.push(DatasetPairRow {
                    sensor: sensor_name.clone(),
                    arf_path,
                    agt_path,
                    stem,
                }),
                (Some(_), None) if missing.report_missing_agt => rows.push(DatasetPairRow {
                    sensor: sensor_name.clone(),
                    arf_path,
                    agt_path: None,
                    stem,
                }),
                (None, Some(_)) if missing.report_missing_arf => rows.push(DatasetPairRow {
                    sensor: sensor_name.clone(),
                    arf_path: None,
                    agt_path,
                    stem,
                }),
                _ => {}
            }
        }
    }

    rows.sort_by(|a, b| (&a.sensor, &a.stem).cmp(&(&b.sensor, &b.stem)));
    Ok(rows)
}

fn list_files_by_stem(dir: &Path, ext: &str) -> io::Result<BTreeMap<String, PathBuf>> {
    let mut map = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(map);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
        {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
            continue;
        };
        map.insert(stem.to_owned(), path);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dataset_layout")
    }

    #[test]
    fn scan_pairs_only_complete_matches() {
        let rows = scan_classic_dataset_pairs(&fixture_root(), MissingSideFlags::default())
            .expect("scan");
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.arf_path.is_some() && r.agt_path.is_some()));
        let stems: BTreeSet<_> = rows.iter().map(|r| r.stem.as_str()).collect();
        assert!(stems.contains("clip_one"));
        assert!(stems.contains("take"));
    }

    #[test]
    fn scan_reports_orphans_when_requested() {
        let rows = scan_classic_dataset_pairs(
            &fixture_root(),
            MissingSideFlags {
                report_missing_agt: true,
                report_missing_arf: true,
            },
        )
        .expect("scan");
        assert_eq!(rows.len(), 4);
        let orphan_arf_only = rows
            .iter()
            .find(|r| r.stem == "orphan_arf")
            .expect("orphan arf");
        assert!(orphan_arf_only.agt_path.is_none());
        let orphan_agt_only = rows
            .iter()
            .find(|r| r.stem == "orphan_agt")
            .expect("orphan agt");
        assert!(orphan_agt_only.arf_path.is_none());
    }
}
