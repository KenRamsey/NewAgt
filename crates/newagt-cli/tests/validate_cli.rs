use std::process::{Command, Stdio};

fn newagt_bin() -> &'static str {
    env!("CARGO_BIN_EXE_newagt")
}

fn fixture(name: &str) -> String {
    format!(
        "{}/../newagt-core/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn validate_valid_fixture_exits_zero() {
    let path = fixture("minimal_agt_tgt.agt");
    let output = Command::new(newagt_bin())
        .args(["validate", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("validation: loadable"));
}

#[test]
fn validate_broken_brace_exits_nonzero() {
    let dir = std::env::temp_dir().join(format!("newagt-validate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("broken.agt");
    std::fs::write(&path, r#"Agt { PrjSect { Name "x""#).expect("write");

    let output = Command::new(newagt_bin())
        .arg("validate")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt validate");

    let _ = std::fs::remove_dir_all(&dir);

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("validation: not loadable"));
}

#[test]
fn validate_json_format() {
    let path = fixture("minimal_agt_tgt.agt");
    let output = Command::new(newagt_bin())
        .args(["validate", "--format", "json", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"loadable\":true"));
    assert!(stdout.contains("\"entries\":["));
}
