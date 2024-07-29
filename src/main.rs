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
    let codepoint = emoji.unicode.split_whitespace().next().expect("No codepoint found");
    let codepoint = codepoint.trim_start_matches("U+");
    let codepoint = u32::from_str_radix(codepoint, 16).expect("Invalid unicode codepoint");
    let emoji_char = char::from_u32(codepoint).expect("Invalid unicode codepoint");

    if printed_emojis.contains(&emoji_char) {
        return; // Skip printing if emoji has already been printed
    }

    println!("{}", emoji_char);
    printed_emojis.insert(emoji_char); // Add printed emoji to the set
    *count += 1;
}

fn main() {
    let search_term = env::args().nth(1).expect("Usage: <search_term>");
    let num_results: usize = env::args().nth(2).map_or(1, |arg| arg.parse().expect("Invalid number of results"));
    let data = include_str!("../emojis.json");
    let emojis: Vec<EmojiRecord> = serde_json::from_str(&data).expect("JSON was not well-formatted");

    let mut printed_emojis = HashSet::new(); // Keep track of printed emojis
    let mut count = 0;

    for emoji in &emojis {
        if emoji.name == search_term {
            print_emoji(emoji, &mut printed_emojis, &mut count);
            return;
        }
        if emoji.keywords.iter().any(|keyword| keyword.contains(&search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count);

            if count >= num_results {
                return;
            }
        }
    }

    // if we didn't find enough emojis, check the definition field
    for emoji in &emojis {
        if let Some(definition) = &emoji.definition {
            if definition.contains(&search_term) {
                print_emoji(emoji, &mut printed_emojis, &mut count);

                if count >= num_results {
                    return;
                }
            }
        }
    }
}
