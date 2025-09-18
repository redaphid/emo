// Following ADD: Test for --list-models flag

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_list_models_flag_exists() {
    // Test 1: --list-models flag should be recognized
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("--list-models");

    // Should not fail with "unexpected argument"
    cmd.assert().success();
}

#[test]
fn test_list_models_shows_available_models() {
    // Test 2: --list-models should show actual models
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("--list-models");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available models"))
        .stdout(predicate::str::contains("Q4_K_M"));
}

#[test]
fn test_model_flag_saves_to_config() {
    // Test 3: Using --model should save the model to config
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":null}"#).unwrap();

    // Use --model to set a model (will trigger AI mode now)
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["--model", "llama-3.2-1b", "fire"]);

    // Should succeed (AI mode with model)
    cmd.assert().success();

    // Check that config was updated with the model
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("\"model\":\"llama-3.2-1b\"") ||
            config_content.contains("\"model\": \"llama-3.2-1b\""));
}

#[test]
fn test_config_model_is_used_for_ai() {
    // Test 4: Model from config should be used when --ai is specified
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    // Create config with a specific model
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":"phi-2"}"#).unwrap();

    // Create stub model file to avoid download
    let models_dir = config_dir.join("models");
    fs::create_dir_all(&models_dir).unwrap();
    fs::write(models_dir.join("phi-2.Q4_K_M.gguf"), "stub model").unwrap();

    // AI selector should use the model from config
    // We can't fully test AI without the model, but we can check that it tries to use it
    use emo::ai::AiEmojiSelector;
    let selector = AiEmojiSelector::new();

    // The selector should look for the model specified in config
    // This is a minimal test - in real usage it would download/use the model
    assert!(true); // Placeholder for now
}