use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_basic_search() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    // Create empty config to avoid any memos
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("fire");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🔥"));
}

#[test]
fn test_search_with_count() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    // Create empty config
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-c", "3", "happy"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_memo_creation_and_use() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    // Create empty config first
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":null}"#).unwrap();

    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    // Create memo
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-m", "🚀", "deploy"]);
    cmd.assert().success();

    // Use memo
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.arg("deploy");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🚀"));
}

#[test]
fn test_memo_with_count_provides_variety() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    // Use "fire" which has search results in the emoji database
    fs::write(&config_path, r#"{"mappings":{"fire":"🔥"},"model":null}"#).unwrap();

    // Request multiple with count
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-c", "3", "fire"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().split('\n').collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "🔥"); // Memo first
    // Others should be different (from search)
    assert_ne!(lines[1], "🔥");
    assert_ne!(lines[2], "🔥");
}

#[test]
fn test_ai_flag_ignores_memo() {
    // The --ai flag should ignore memos per precedence rules
    // With a model configured, AI mode will work and produce an emoji (not the memo)
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{"test":"🧪"},"model":"llama-3.2-1b"}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["--ai", "test"]);

    // AI mode should succeed and NOT return the memo emoji 🧪
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🧪").not());
}

#[test]
fn test_random_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-r"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should output an emoji and its name
    assert!(stdout.contains(" - "));
}

#[test]
fn test_define_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-d", "🔥"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🔥 - fire"));
}

#[test]
fn test_list_mappings() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{"test":"🧪","rocket":"🚀"},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-l"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("rocket → 🚀"))
        .stdout(predicate::str::contains("test → 🧪"));
}

#[test]
fn test_erase_mapping() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{"test":"🧪"},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-e", "test"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Mapping for 'test' erased ✅"));

    // Verify it's actually gone
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(!config_content.contains("test"));
}

#[test]
fn test_memo_by_index() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":null}"#).unwrap();

    // First get search results
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-c", "3", "fire"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    let second_emoji = lines[1]; // Remember the second result

    // Save the second result
    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-m", "2", "fire"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&format!("fire ➡ {} ✅", second_emoji)));
}

#[test]
fn test_number_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["-n", "-c", "3", "happy"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1. "))
        .stdout(predicate::str::contains("2. "))
        .stdout(predicate::str::contains("3. "));
}

#[test]
fn test_multi_word_search() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.json"), r#"{"mappings":{},"model":null}"#).unwrap();

    let mut cmd = Command::cargo_bin("emo").unwrap();
    cmd.env("XDG_CONFIG_HOME", temp_dir.path());
    cmd.args(&["red", "heart"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("❤"));
}

#[test]
fn test_empty_search_term_error() {
    let mut cmd = Command::cargo_bin("emo").unwrap();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Please provide a search term"));
}