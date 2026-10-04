use std::process::{Command, Stdio};

fn newagt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_newagt")
}

fn dataset_fixture_root() -> String {
    format!(
        "{}/../newagt-core/tests/fixtures/dataset_layout",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn pairs_lists_complete_matches_only_by_default() {
    let root = dataset_fixture_root();
    let output = Command::new(newagt_bin())
        .args(["pairs", "--dataset-root", &root])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt pairs");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|l| {
        l.contains('\t') && (l.ends_with("clip_one") || l.ends_with("take"))
    }));
    assert!(stdout.contains("sensor_alpha"));
    assert!(stdout.contains("sensor_beta"));
    assert!(!stdout.contains("orphan_arf"));
    assert!(!stdout.contains("orphan_agt"));
}

#[test]
fn pairs_missing_both_includes_orphans() {
    let root = dataset_fixture_root();
    let output = Command::new(newagt_bin())
        .args(["pairs", "--dataset-root", &root, "--missing", "both"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt pairs");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.lines().count(), 4);
    assert!(stdout.contains("orphan_arf"));
    assert!(stdout.contains("orphan_agt"));
}
