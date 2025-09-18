// Following ADD: Start with existence tests

use emo::generators::{MemoGenerator, EmojiGenerator};
use std::collections::HashMap;

#[test]
fn memo_generator_exists() {
    // Test 1: MemoGenerator type exists
    let _ = MemoGenerator::new();
}

#[test]
fn memo_generator_implements_trait() {
    // Test 2: MemoGenerator implements EmojiGenerator
    let gen = MemoGenerator::new();
    let _result = gen.generate("test");
}

#[test]
fn memo_generator_with_mappings_returns_memo() {
    // Test 3: MemoGenerator with a memo returns the right emoji
    let mut mappings = HashMap::new();
    mappings.insert("deploy".to_string(), '🚀');

    let gen = MemoGenerator::with_mappings(mappings);
    let result = gen.generate("deploy").unwrap();
    assert_eq!(result, "🚀");
}

#[test]
fn memo_generator_with_different_mapping() {
    // Test 4: Force generalization - not just "deploy"
    let mut mappings = HashMap::new();
    mappings.insert("fire".to_string(), '🔥');

    let gen = MemoGenerator::with_mappings(mappings);
    let result = gen.generate("fire").unwrap();
    assert_eq!(result, "🔥");
}