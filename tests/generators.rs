use emo::generators::{EmojiGenerator, SearchGenerator};

#[test]
fn emoji_generator_trait_exists() {
    // This test will fail until we define the trait
    // Just checking it compiles is the test
}

#[test]
fn emoji_generator_has_generate_method() {
    struct TestGenerator;
    impl EmojiGenerator for TestGenerator {
        fn generate(&self, _input: &str) -> Result<String, emo::error::EmoError> {
            Ok("test".to_string())
        }
    }
}

#[test]
fn search_generator_exists() {
    let _generator = SearchGenerator::new();
}

#[test]
fn search_generator_implements_trait() {
    let generator = SearchGenerator::new();
    let _result = generator.generate("fire");
}

#[test]
fn search_generator_returns_fire_emoji_for_fire() {
    let generator = SearchGenerator::new();
    let result = generator.generate("fire").unwrap();
    assert_eq!(result, "🔥");
}

#[test]
fn search_generator_returns_emoji_for_happy() {
    let generator = SearchGenerator::new();
    let result = generator.generate("happy").unwrap();
    // Should return some emoji, not a specific one since search can vary
    assert!(!result.is_empty());
    assert!(result.chars().all(|c| c as u32 > 127)); // Non-ASCII means emoji
}