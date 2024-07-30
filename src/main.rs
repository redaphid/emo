use clap::{Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[derive(Parser)]
#[command(version = None)]
struct Cli {
    #[arg(short, long, default_value_t = 1, help = "number of results to show")]
    count: usize,
    #[arg(short, long, default_value_t = false, help = "show emoji names")]
    name:bool,
    search_term: String,
    #[arg(short, long, default_value_t = false, help = "define the specified emoji")]
    define:bool,
}
fn main() {
    let cmd = Cli::parse();
    let search_term = &cmd.search_term;
    let show_name = cmd.name;
    let num_results = cmd.count;
    let define = cmd.define;
    let mut count = 0;

    let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json")).unwrap();

    if define {
        // get search term as unicode code. iterate through the emojis, split the 'unicode' property and compare the first character with the search term
        // if it matches, print the emoji
        for emoji in &emojis {
           for unicode in emoji.unicode.split_whitespace() {
               let emoji_char = char::from_u32(u32::from_str_radix(unicode.trim_start_matches("U+"), 16).unwrap()).unwrap();
               if emoji_char == search_term.chars().next().unwrap(){
                   println!("{} - {}", emoji_char, emoji.definition.as_ref().unwrap());
               }
               count += 1;
               if count >= num_results { return; }
           }
        }
        return;
    }

    let mut printed_emojis = HashSet::new();

    for emoji in &emojis {
        if emoji.name.contains(search_term) || emoji.keywords.iter().any(|k| k.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count, show_name);
            if count >= num_results { return; }
        }
    }

    for emoji in &emojis {
        if emoji.definition.as_deref().map_or(false, |d| d.contains(search_term)) {
            print_emoji(emoji, &mut printed_emojis, &mut count, show_name);
            if count >= num_results { return; }
        }
    }
}
