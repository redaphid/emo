use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_vampire_pear_sentence_consistency() {
    // Test that specific prompt consistently includes vampire and pear emojis
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    // Set up config with model
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":"llama-3.2-1b"}"#).unwrap();

    let prompt = "A man, lovestruck, thinking about a vampire eating a pear";

    // Run multiple times and check for consistency
    for run in 1..=3 {  // Reduce runs to speed up test
        let mut cmd = Command::cargo_bin("emo").unwrap();
        cmd.env("XDG_CONFIG_HOME", temp_dir.path());
        cmd.args(&["--ai", "--sentence", "10", prompt]);

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Check for vampire emoji (🦇 or 🧛) and pear (🍐)
        assert!(
            stdout.contains("🦇") || stdout.contains("🧛") || stdout.contains("🧛‍♂️") || stdout.contains("🧛‍♀️"),
            "Run {} missing vampire emoji in: {}", run, stdout
        );
        assert!(
            stdout.contains("🍐"),
            "Run {} missing pear emoji in: {}", run, stdout
        );
    }
}

#[test]
fn test_sentence_length() {
    // Test that sentence generation produces requested number of emojis
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("emo");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.json");
    fs::write(&config_path, r#"{"mappings":{},"model":"llama-3.2-1b"}"#).unwrap();

    for length in [5, 10, 15] {
        let mut cmd = Command::cargo_bin("emo").unwrap();
        cmd.env("XDG_CONFIG_HOME", temp_dir.path());
        cmd.args(&["--ai", "--sentence", &length.to_string(), "happy birthday"]);

        let output = cmd.output().unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();

        // Count emoji characters (this is approximate - some emojis are multi-codepoint)
        let emoji_count = stdout.chars().filter(|c| {
            matches!((*c as u32),
                0x1F300..=0x1F9FF | // Emoticons & misc
                0x2600..=0x26FF |   // Misc symbols
                0x2700..=0x27BF |   // Dingbats
                0x1F000..=0x1F02F | // Mahjong/Domino
                0x1FA70..=0x1FAFF   // More symbols
            )
        }).count();

        // Allow some flexibility due to multi-codepoint emojis
        assert!(
            emoji_count >= length - 2 && emoji_count <= length + 2,
            "Expected ~{} emojis, got {} in: {}", length, emoji_count, stdout
        );
    }
}