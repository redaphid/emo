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

fn print_emoji(emoji: &EmojiRecord, printed_emojis: &mut HashSet<char>, count: &mut usize, show_name: bool) {
    let emoji_char = char::from_u32(u32::from_str_radix(emoji.unicode.split_whitespace().next().unwrap().trim_start_matches("U+"), 16).unwrap()).unwrap();

    if printed_emojis.insert(emoji_char) {
        if show_name {
            println!("{} - {}", emoji_char, emoji.name);
        } else {
            println!("{}", emoji_char);
        }
        *count += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let show_name = args.contains(&"-n".to_string());

    // Extract the count if provided
    let count_index = args.iter().position(|arg| arg == "-c").and_then(|index| {
        args.get(index + 1).and_then(|count_str| count_str.parse().ok())
    }).unwrap_or(1);

    // Determine search term, ignoring flags
    let search_term = args.iter().skip(1).find(|&&ref arg| !arg.starts_with('-')).expect("Usage: <search_term> [-n] [-c <count>]");

    let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json")).unwrap();

    let mut printed_emojis = HashSet::new();
    let mut count = 0;

    for emoji in &emojis {
        if emoji.name == *search_term || emoji.keywords.iter().any(|k| k.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count, show_name);
            if count >= count_index { return; }
        }
    }

    for emoji in &emojis {
        if emoji.definition.as_deref().map_or(false, |d| d.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count, show_name);
            if count >= count_index { return; }
        }
    }
}
