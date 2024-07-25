use serde::{Deserialize, Serialize};
use regex::Regex;
use std::env;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct EmojiRecord {
    // category: String,
    keywords: Vec<String>,
    // definition: String,
    unicode: String,
    name: String,
    shortcode: Option<String>,
}

fn main() {
    let search_term = env::args().nth(1).expect("Usage: <search_term>");
    let file_path = "emojis.json";

    // Read the JSON file
    let data = fs::read_to_string(file_path).expect("Unable to read file");
    // Parse the JSON data
    let emojis: Vec<EmojiRecord> = serde_json::from_str(&data).expect("JSON was not well-formatted");

    // Create the regex for fuzzy search
    let search_regex = Regex::new(&format!("(?i){}", search_term)).expect("Invalid regex");

    // Search for the keyword and print the corresponding emoji
    for emoji in emojis {
        if emoji.keywords.iter().any(|keyword| search_regex.is_match(keyword)) {
            let codepoints: Vec<String> = emoji.unicode.split_whitespace().map(|codepoint| {
                let codepoint = codepoint.trim_start_matches("U+");
                let codepoint = u32::from_str_radix(codepoint, 16).expect("Invalid unicode codepoint");
                char::from_u32(codepoint).expect("Invalid unicode codepoint").to_string()
            }).collect();
            let emoji_string = codepoints.join("\u{200D}"); // Zero-width joiner
            println!("{}", emoji_string);
        }
    }
}
