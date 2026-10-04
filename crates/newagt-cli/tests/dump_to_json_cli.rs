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

#[test]
fn info_directory_emits_one_line_per_agt() {
    let root = std::env::temp_dir().join(format!("newagt-info-dir-{}", std::process::id()));
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).expect("temp dirs");
    let src = fixture("minimal_agt_tgt.agt");
    std::fs::copy(&src, root.join("top.agt")).expect("copy top");
    std::fs::copy(&src, nested.join("deep.agt")).expect("copy nested");

    let output = Command::new(newagt_bin())
        .args(["info", root.to_str().expect("utf8 path")])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt info on directory");

    let _ = std::fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        assert!(line.contains("path:"));
        assert!(line.contains("frames:"));
        assert!(line.contains("sections:"));
        assert!(line.contains("parse: ok"));
    }
}

#[test]
fn info_directory_limit_caps_output_lines() {
    let root = std::env::temp_dir().join(format!("newagt-info-limit-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("temp dir");
    let src = fixture("minimal_agt_tgt.agt");
    for name in ["a.agt", "b.agt", "c.agt"] {
        std::fs::copy(&src, root.join(name)).expect("copy agt");
    }

    let output = Command::new(newagt_bin())
        .args([
            "info",
            root.to_str().expect("utf8 path"),
            "--limit",
            "2",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt info --limit");

    let _ = std::fs::remove_dir_all(&root);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).lines().count(),
        2
    );
}

#[test]
fn info_directory_prints_stderr_batch_summary() {
    let root = std::env::temp_dir().join(format!("newagt-info-summary-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("temp dir");
    let src = fixture("minimal_agt_tgt.agt");
    std::fs::copy(&src, root.join("one.agt")).expect("copy agt");

    let output = Command::new(newagt_bin())
        .args(["info", root.to_str().expect("utf8 path")])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt info on directory");

    let _ = std::fs::remove_dir_all(&root);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("scanned 1 files"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("parse ok 1, fail 0"));
    assert!(stderr.contains("total frames"));
}

#[test]
fn info_directory_progress_flag_emits_stderr_lines() {
    let root = std::env::temp_dir().join(format!("newagt-info-progress-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("temp dir");
    let src = fixture("minimal_agt_tgt.agt");
    for name in ["a.agt", "b.agt"] {
        std::fs::copy(&src, root.join(name)).expect("copy agt");
    }

    let output = Command::new(newagt_bin())
        .args([
            "info",
            root.to_str().expect("utf8 path"),
            "--progress",
            "1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run newagt info --progress");

    let _ = std::fs::remove_dir_all(&root);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("progress 1/2"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("progress 2/2"));
}
