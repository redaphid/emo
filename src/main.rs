use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct EmojiRecord {
    keywords: Vec<String>,
    unicode: String,
    name: String,
    shortcode: Option<String>,
}

fn main() {
    let search_term = env::args().nth(1).expect("Usage: <search_term>");
    let num_results: usize = env::args().nth(2).map_or(1, |arg| arg.parse().expect("Invalid number of results"));
    let data = include_str!("../emojis.json");
    let emojis: Vec<EmojiRecord> = serde_json::from_str(&data).expect("JSON was not well-formatted");

    let mut printed_emojis = HashSet::new(); // Keep track of printed emojis

    let mut count = 0;
    for emoji in emojis {
        if emoji.keywords.iter().any(|keyword| keyword == &search_term) {
            let codepoint = emoji.unicode.split_whitespace().next().expect("No codepoint found");
            let codepoint = codepoint.trim_start_matches("U+");
            let codepoint = u32::from_str_radix(codepoint, 16).expect("Invalid unicode codepoint");
            let emoji_char = char::from_u32(codepoint).expect("Invalid unicode codepoint");

            if printed_emojis.contains(&emoji_char) {
                continue; // Skip printing if emoji has already been printed
            }

            println!("{}", emoji_char);
            printed_emojis.insert(emoji_char); // Add printed emoji to the set
            count += 1;

            if count >= num_results {
                return;
            }
        }
    }
}
