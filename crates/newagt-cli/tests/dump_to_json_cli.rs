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
fn dump_valid_fixture_exits_zero() {
    let path = fixture("minimal_agt_tgt.agt");
    let output = Command::new(newagt_bin())
        .args(["dump", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt dump");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TgtType \"M1A1\""));
    assert!(stdout.contains("PixLoc 200 230"));
}

#[test]
fn dump_parse_failure_exits_one() {
    let dir = std::env::temp_dir().join(format!("newagt-dump-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("broken.agt");
    std::fs::write(&path, r#"Agt { PrjSect { Name "x""#).expect("write");

    let output = Command::new(newagt_bin())
        .arg("dump")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt dump");

    let _ = std::fs::remove_dir_all(&dir);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn to_json_schema_and_round_trip() {
    let path = fixture("minimal_agt_tgt.agt");
    let output = Command::new(newagt_bin())
        .args(["to-json", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt to-json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid json");
    assert_eq!(v["schema"], "newagt.schema.v1");
    assert_eq!(v["document"]["kind"], "Agt");
}

#[test]
fn info_prints_section_counts() {
    let path = fixture("minimal_agt_tgt.agt");
    let output = Command::new(newagt_bin())
        .args(["info", &path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt info");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("profile: agtj"));
    assert!(stdout.contains("TgtSect=1"));
    assert!(stdout.contains("Tgt=1"));
    assert!(stdout.contains("frames: count="));
}
