use serde::{Deserialize, Serialize};
use std::env;

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
    let num_results: usize = env::args().nth(2).map_or(usize::MAX, |arg| arg.parse().expect("Invalid number of results"));
    // Read the JSON file
    let data = include_str!("../emojis.json");
    // Parse the JSON data
    let emojis: Vec<EmojiRecord> = serde_json::from_str(&data).expect("JSON was not well-formatted");

    // Search for the keyword and print the corresponding emoji
    let mut count = 0;
    for emoji in emojis {
        if emoji.keywords.iter().any(|keyword| keyword.contains(&search_term)) {
            let codepoints: Vec<String> = emoji.unicode.split_whitespace().map(|codepoint| {
                let codepoint = codepoint.trim_start_matches("U+");
                let codepoint = u32::from_str_radix(codepoint, 16).expect("Invalid unicode codepoint");
                char::from_u32(codepoint).expect("Invalid unicode codepoint").to_string()
            }).collect();
            let emoji_string = codepoints.join("\u{200D}"); // Zero-width joiner
            println!("{}", emoji_string);
            count+=1;
            if count >= num_results {
                return;
            }
        }
    }
}
