pub mod ai;
pub mod error;
pub mod generators;

use error::{EmoError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct EmojiRecord {
    pub keywords: Vec<String>,
    pub unicode: String,
    pub name: String,
    pub shortcode: Option<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmojiMappings {
    pub mappings: HashMap<String, char>,
    pub model: Option<String>,  // Optional model in llama/ollama format
}

impl Default for EmojiMappings {
    fn default() -> Self {
        // Parse the bundled default config
        serde_json::from_str(include_str!("../default_config.json"))
            .expect("Default config should be valid JSON")
    }
}

impl EmojiMappings {
    fn get_config_dir() -> Result<std::path::PathBuf> {
        // Check for XDG_CONFIG_HOME first (for testing and custom configs)
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(std::path::PathBuf::from(xdg_config));
        }

        // Fall back to system default
        dirs::config_dir().ok_or_else(|| {
            EmoError::ConfigError("Could not determine config directory".to_string())
        })
    }

    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let path = config_dir.join("emo").join("config.json");

        if path.exists() {
            let file = std::fs::File::open(path)?;
            Ok(serde_json::from_reader(file)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let emo_dir = config_dir.join("emo");
        std::fs::create_dir_all(&emo_dir)?;
        let path = emo_dir.join("config.json");
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}

use std::sync::OnceLock;

static EMOJIS: OnceLock<Vec<EmojiRecord>> = OnceLock::new();

pub fn load_emojis() -> Result<&'static Vec<EmojiRecord>> {
    Ok(EMOJIS.get_or_init(|| {
        let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json"))
            .expect("Failed to parse emoji data");
        emojis
            .into_iter()
            .filter(|e| !e.unicode.contains(' '))
            .collect()
    }))
}

pub fn to_char(emoji: &EmojiRecord) -> Result<char> {
    let unicode_part = emoji
        .unicode
        .split_whitespace()
        .next()
        .ok_or_else(|| EmoError::InvalidInput(format!("Invalid unicode: {}", emoji.unicode)))?;

    let hex_str = unicode_part.trim_start_matches("U+");
    let code_point = u32::from_str_radix(hex_str, 16)
        .map_err(|_| EmoError::InvalidInput(format!("Invalid hex code: {}", hex_str)))?;

    char::from_u32(code_point)
        .ok_or_else(|| EmoError::InvalidInput(format!("Invalid code point: {}", code_point)))
}

fn is_exact_word_match(text: &str, search: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric())
        .any(|word| word.to_lowercase() == search)
}

use std::collections::BTreeMap;

pub struct SearchIndex {
    name_index: BTreeMap<String, Vec<usize>>,
    keyword_index: BTreeMap<String, Vec<usize>>,
}

impl SearchIndex {
    pub fn build(emojis: &[EmojiRecord]) -> Self {
        let mut name_index = BTreeMap::new();
        let mut keyword_index = BTreeMap::new();

        for (idx, emoji) in emojis.iter().enumerate() {
            // Index name words
            for word in emoji.name.split_whitespace() {
                name_index
                    .entry(word.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }

            // Index keyword words
            for keyword in &emoji.keywords {
                for word in keyword.split_whitespace() {
                    keyword_index
                        .entry(word.to_lowercase())
                        .or_insert_with(Vec::new)
                        .push(idx);
                }
            }
        }

        Self {
            name_index,
            keyword_index,
        }
    }
}

static SEARCH_INDEX: OnceLock<SearchIndex> = OnceLock::new();

pub fn get_search_index(emojis: &[EmojiRecord]) -> &'static SearchIndex {
    SEARCH_INDEX.get_or_init(|| SearchIndex::build(emojis))
}

pub fn search<'a>(
    emojis: &'a [EmojiRecord],
    search_term: &str,
    num_results: usize,
) -> Vec<(char, &'a EmojiRecord)> {
    let search_words: Vec<String> = search_term
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let all_words_match = |text: &str, exact: bool| {
        let text = text.to_lowercase();
        search_words.iter().all(|word| {
            if exact {
                is_exact_word_match(&text, word)
            } else {
                text.contains(word)
            }
        })
    };

    let predicates: Vec<Box<dyn Fn(&EmojiRecord) -> bool>> = vec![
        Box::new(|e| e.name.to_lowercase() == search_term.to_lowercase()),
        Box::new(|e| all_words_match(&e.name, true)),
        Box::new(|e| e.keywords.iter().any(|k| all_words_match(k, true))),
        Box::new(|e| all_words_match(&e.name, false)),
        Box::new(|e| e.keywords.iter().any(|k| all_words_match(k, false))),
        Box::new(|e| {
            e.definition
                .as_ref()
                .map_or(false, |d| all_words_match(d, false))
        }),
    ];

    for predicate in predicates {
        for emoji in emojis {
            if !predicate(emoji) {
                continue;
            }
            let c = match to_char(emoji) {
                Ok(ch) => ch,
                Err(_) => continue,
            };
            if !seen.insert(c) {
                continue;
            }
            results.push((c, emoji));
            if results.len() >= num_results {
                return results;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_char() {
        let emoji = EmojiRecord {
            keywords: vec!["test".to_string()],
            unicode: "U+1F600".to_string(),
            name: "grinning face".to_string(),
            shortcode: None,
            definition: None,
        };
        assert_eq!(to_char(&emoji).unwrap(), '😀');
    }

    #[test]
    fn test_is_exact_word_match() {
        assert!(is_exact_word_match("hello world", "hello"));
        assert!(is_exact_word_match("hello world", "world"));
        assert!(!is_exact_word_match("hello world", "hell"));
        assert!(!is_exact_word_match("hello world", "orld"));
    }

    #[test]
    fn test_search_by_name() {
        let emojis = vec![
            EmojiRecord {
                keywords: vec!["happy".to_string()],
                unicode: "U+1F600".to_string(),
                name: "grinning face".to_string(),
                shortcode: None,
                definition: None,
            },
            EmojiRecord {
                keywords: vec!["sad".to_string()],
                unicode: "U+1F622".to_string(),
                name: "crying face".to_string(),
                shortcode: None,
                definition: None,
            },
        ];

        let results = search(&emojis, "grinning", 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "grinning face");
    }

    #[test]
    fn test_search_by_keyword() {
        let emojis = vec![EmojiRecord {
            keywords: vec!["happy".to_string(), "smile".to_string()],
            unicode: "U+1F600".to_string(),
            name: "grinning face".to_string(),
            shortcode: None,
            definition: None,
        }];

        let results = search(&emojis, "happy", 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.name, "grinning face");
    }

    #[test]
    fn test_emoji_mappings_default() {
        let mappings = EmojiMappings::default();
        assert!(mappings.mappings.is_empty());
    }
}
