use emo::EmojiMappings;

#[test]
fn test_default_config_has_empty_mappings() {
    let config = EmojiMappings::default();
    assert!(config.mappings.is_empty());
}

#[test]
fn test_default_config_has_null_model() {
    let config = EmojiMappings::default();
    assert!(config.model.is_none());
}

#[test]
fn test_default_config_matches_bundled_json() {
    let config = EmojiMappings::default();
    let expected_json = include_str!("../default_config.json");
    let expected: EmojiMappings = serde_json::from_str(expected_json).unwrap();

    assert_eq!(config.mappings.len(), expected.mappings.len());
    assert_eq!(config.model, expected.model);
}