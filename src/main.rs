use clap::{value_parser, Arg, Command};
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

fn main() {
    let matches = Command::new("emoji_search")
        .version("1.0")
        .arg(Arg::new("search_term")
            .help("The term to search for")
            .required(true)
            .index(1))
        .arg(Arg::new("name")
            .short('n')
            .long("name")
            .help("Show emoji names"))
        .arg(Arg::new("count")
            .short('c')
            .value_parser(value_parser!(usize))
            .long("count")
            .help("Number of results to show")
            .default_value("1")
          )
        .get_matches();

    let show_name = matches.contains_id("name");
    let num_results = *matches.get_one::<usize>("count").unwrap();
    let search_term = matches.get_one::<String>("search_term").unwrap();

    let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json")).unwrap();

    let mut printed_emojis = HashSet::new();
    let mut count = 0;

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
