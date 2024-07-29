use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct EmojiRecord {
    keywords: Vec<String>,
    unicode: String,
    name: String,
    shortcode: Option<String>,
    definition: Option<String>,
}

fn print_emoji(emoji: &EmojiRecord, printed_emojis: &mut HashSet<char>, count: &mut usize) {
    let emoji_char = char::from_u32(u32::from_str_radix(emoji.unicode.split_whitespace().next().unwrap().trim_start_matches("U+"), 16).unwrap()).unwrap();

    if printed_emojis.insert(emoji_char) {
        println!("{}", emoji_char);
        *count += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let search_term = &args[1];
    let num_results: usize = args.get(2).and_then(|arg| arg.parse().ok()).unwrap_or(1);
    let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json")).unwrap();

    let mut printed_emojis = HashSet::new();
    let mut count = 0;

    for emoji in &emojis {
        if emoji.name == *search_term || emoji.keywords.iter().any(|k| k.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count);
            if count >= num_results { return; }
        }
    }

    for emoji in &emojis {
        if emoji.definition.as_deref().map_or(false, |d| d.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count);
            if count >= num_results { return; }
        }
    }
}
