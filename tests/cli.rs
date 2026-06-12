//! End-to-end CLI tests: exit codes and stream routing (Phase 1.2).
//!
//! Contract: 0 = success, 1 = script failure (parse/runtime/missing file),
//! 2 = CLI usage error. Errors go to stderr, program output to stdout.

use assert_cmd::Command;

fn shik() -> Command {
    Command::cargo_bin("shik").unwrap()
}

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

#[test]
fn successful_script_exits_zero() {
    shik()
        .arg(fixture("ok.shk"))
        .assert()
        .success()
        .stdout("hello from fixture\n")
        .stderr("");
}

#[test]
fn parse_error_exits_one_on_stderr() {
    let assert = shik().arg(fixture("parse_error.shk")).assert().code(1);
    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn runtime_error_exits_one_on_stderr() {
    let assert = shik().arg(fixture("runtime_error.shk")).assert().code(1);
    let output = assert.get_output();
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn missing_file_exits_one_with_message() {
    let assert = shik().arg(fixture("does_not_exist.shk")).assert().code(1);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("cannot open") && stderr.contains("does_not_exist.shk"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn unknown_flag_exits_two() {
    let assert = shik().arg("--frobnicate").assert().code(2);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("--frobnicate"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn ast_flag_prints_ast_without_evaluating() {
    let assert = shik()
        .arg("--ast")
        .arg(fixture("ok.shk"))
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    // AST debug output, not the script's print output.
    assert!(
        !stdout.contains("hello from fixture"),
        "script was evaluated: {stdout}"
    );
    assert!(stdout.contains("Statement"), "no AST in stdout: {stdout}");
}

#[test]
fn ast_flag_on_broken_file_exits_one() {
    shik()
        .arg("--ast")
        .arg(fixture("parse_error.shk"))
        .assert()
        .code(1);
}
