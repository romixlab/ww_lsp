//! The CLI as a script uses it: the built binary, without colour, without the usage record and without the
//! `CARGO_*` variables of the test run (the binary must not depend on them).

use std::path::Path;
use std::process::{Command, Output};

fn fixture() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ww_lsp_core/tests/fixtures/psu_api.rs")
        .display()
        .to_string()
}

fn ww_lsp(args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ww_lsp"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("CARGO_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("WW_LSP_USAGE", "off")
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn hints_of_a_file() {
    let out = ww_lsp(&["hints", &fixture()]);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let first = stdout.lines().next().unwrap();
    assert_eq!(
        first,
        format!(
            "{}:22:48  mV · i16 · ±32.767 V  (property v_set)",
            fixture()
        )
    );
    assert_eq!(stdout.lines().count(), 11);
}

#[test]
fn hover_at_a_position() {
    let out = ww_lsp(&["hover", &format!("{}:30:25", fixture())]);
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("**Q15 · −1..1**")
    );
    let none = ww_lsp(&["hover", &format!("{}:1:1", fixture())]);
    assert_eq!(none.status.code(), Some(1));
}

#[test]
fn a_file_that_is_not_rust_fails_with_its_place() {
    let dir = std::env::temp_dir().join(format!("ww_lsp_cli_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.rs");
    std::fs::write(&bad, "#[ww_trait]\ntrait Psu {").unwrap();
    let out = ww_lsp(&["hints", bad.to_str().unwrap()]);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.starts_with("error: "), "{stderr}");
    assert!(stderr.contains("bad.rs:2:11: not Rust"), "{stderr}");
}

#[test]
fn version_names_the_commit() {
    let out = ww_lsp(&["--version"]);
    let v = String::from_utf8(out.stdout).unwrap();
    assert!(
        v.starts_with(&format!("ww_lsp {} (", env!("CARGO_PKG_VERSION"))),
        "{v}"
    );
    assert!(v.contains(", commit "), "{v}");
}
