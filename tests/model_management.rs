// Following ADD: Start with existence test for --list-models flag

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn list_models_flag_exists() {
    // Test 1: --list-models flag should be recognized
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("--list-models");

    // Should succeed now that it's implemented
    cmd.assert().success();
}

#[test]
fn list_models_shows_models() {
    // Test 2: --list-models should output a list of models
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("--list-models");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available models:"));
}

#[test]
fn list_models_shows_real_models() {
    // Test 3: --list-models should show actual downloadable models
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("--list-models");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("llama"))  // Should have some Llama models
        .stdout(predicate::str::contains("Q4_K_M")); // Should mention Q4_K_M quantization
}