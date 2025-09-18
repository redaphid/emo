use crate::error::{Result, EmoError};
use crate::{load_emojis, search};
use std::collections::HashMap;

pub trait EmojiGenerator {
    fn generate(&self, input: &str) -> Result<String>;
}

pub struct SearchGenerator;

impl SearchGenerator {
    pub fn new() -> Self {
        SearchGenerator
    }
}

impl EmojiGenerator for SearchGenerator {
    fn generate(&self, input: &str) -> Result<String> {
        let emojis = load_emojis()?;
        let results = search(emojis, input, 1);

        if results.is_empty() {
            return Err(EmoError::InvalidInput(format!("No emoji found for '{}'", input)));
        }

        Ok(results[0].0.to_string())
    }
}

// Minimal MemoGenerator to pass existence test
pub struct MemoGenerator {
    mappings: HashMap<String, char>,
}

impl MemoGenerator {
    pub fn new() -> Self {
        MemoGenerator {
            mappings: HashMap::new(),
        }
    }

    pub fn with_mappings(mappings: HashMap<String, char>) -> Self {
        MemoGenerator { mappings }
    }
}

impl EmojiGenerator for MemoGenerator {
    fn generate(&self, input: &str) -> Result<String> {
        // Now use actual mappings
        match self.mappings.get(input) {
            Some(emoji) => Ok(emoji.to_string()),
            None => Err(EmoError::InvalidInput("No memo found".to_string())),
        }
    }
}