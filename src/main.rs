use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
struct EmojiRecord {
    keywords: Vec<String>,
    unicode: String,
    name: String,
    shortcode: Option<String>,
    definition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct EmojiMappings {
    mappings: std::collections::HashMap<String, String>,
}

impl EmojiMappings {
    fn load() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_default()
            .join("emo")
            .join("mappings.json");

        if let Ok(file) = std::fs::File::open(path) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let config_dir = dirs::config_dir().unwrap_or_default().join("emo");
        std::fs::create_dir_all(&config_dir)?;
        let path = config_dir.join("mappings.json");
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}

fn try_print(s: &str) {
    let _ = writeln!(std::io::stdout(), "{}", s);
}

#[derive(Parser)]
#[command(version = None)]
struct Cli {
    #[arg(short, long, default_value_t = 1, help = "number of results to show")]
    count: usize,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "define the specified emoji"
    )]
    define: bool,
    #[arg(
        short = 's',
        long,
        help = "save a mapping for the search term to a specific emoji"
    )]
    save: Option<String>,
    #[arg(short = 'n', long, help = "display the number of a given emoji result")]
    number: bool,
    #[arg(trailing_var_arg = true)]
    search_terms: Vec<String>,
}

// Helper function to check if it's an exact word match
fn is_exact_word_match(text: &str, search: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric())
        .any(|word| word.to_lowercase() == search)
}

// Helper to convert emoji to char
fn to_char(emoji: &EmojiRecord) -> char {
    char::from_u32(
        u32::from_str_radix(
            emoji
                .unicode
                .split_whitespace()
                .next()
                .unwrap()
                .trim_start_matches("U+"),
            16,
        )
        .unwrap(),
    )
    .unwrap()
}

fn find_emojis<'a>(
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

    // Helper to check if all search words match a predicate
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

    // List of search predicates in priority order
    let predicates: Vec<Box<dyn Fn(&EmojiRecord) -> bool>> = vec![
        // Priority 1: Exact name match (all words in order)
        Box::new(|e| e.name.to_lowercase() == search_term.to_lowercase()),
        // Priority 2: All words match name exactly
        Box::new(|e| all_words_match(&e.name, true)),
        // Priority 3: All words match keywords exactly
        Box::new(|e| e.keywords.iter().any(|k| all_words_match(k, true))),
        // Priority 4: All words contained in name
        Box::new(|e| all_words_match(&e.name, false)),
        // Priority 5: All words contained in keywords
        Box::new(|e| e.keywords.iter().any(|k| all_words_match(k, false))),
        // Priority 6: All words contained in definition
        Box::new(|e| {
            e.definition
                .as_ref()
                .map_or(false, |d| all_words_match(d, false))
        }),
    ];

    // Try each predicate in order until we have enough results
    for predicate in predicates {
        for emoji in emojis {
            if predicate(emoji) {
                let c = to_char(emoji);
                if seen.insert(c) {
                    results.push((c, emoji));
                    if results.len() >= num_results {
                        return results;
                    }
                }
            }
        }
    }

    results
}

fn print_emojis(results: &[(char, &EmojiRecord)], show_number: bool) {
    for (i, (emoji_char, emoji)) in results.iter().enumerate() {
        let description = emoji.definition.as_deref().unwrap_or("");
        let prefix = if show_number {
            format!("{}. ", i + 1)
        } else {
            String::new()
        };

        try_print(&format!(
            "{}{} - {}. {}",
            prefix, emoji_char, emoji.name, description
        ));
    }
}

fn search_emojis(emojis: &[EmojiRecord], search_term: &str, num_results: usize, show_number: bool) {
    let results = find_emojis(emojis, search_term, num_results);
    print_emojis(&results, show_number);
}

fn main() {
    if let Err(e) = try_main() {
        if let Some(errno) = e.raw_os_error() {
            if errno == 32 {
                // EPIPE
                std::process::exit(0);
            }
        }
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn try_main() -> std::io::Result<()> {
    let cmd = Cli::parse();
    if cmd.search_terms.is_empty() {
        eprintln!("Error: Please provide a search term");
        std::process::exit(1);
    }
    let search_term = &cmd.search_terms.join(" ");
    let num_results = cmd.count;
    let define = cmd.define;
    let show_number = cmd.number;

    let mut mappings = EmojiMappings::load();

    // Handle recording a new mapping
    if let Some(emoji) = cmd.save {
        let emoji_clone = emoji.clone();
        mappings.mappings.insert(search_term.clone(), emoji);
        mappings.save()?;
        try_print(&format!("{} ➡ {} ✅", search_term, emoji_clone));
        return Ok(());
    }

    // Check for custom mapping first
    if let Some(emoji) = mappings.mappings.get(search_term) {
        try_print(emoji);
        return Ok(());
    }

    let emojis: Vec<EmojiRecord> = serde_json::from_str(include_str!("../emojis.json")).unwrap();
    let emojis: Vec<EmojiRecord> = emojis
        .into_iter()
        .filter(|e| !e.unicode.contains(' '))
        .collect();

    if define {
        for emoji in &emojis {
            let emoji_char = char::from_u32(
                u32::from_str_radix(emoji.unicode.trim_start_matches("U+"), 16).unwrap(),
            )
            .unwrap();
            if emoji_char == search_term.chars().next().unwrap() {
                try_print(&format!(
                    "{} - {}",
                    emoji_char,
                    emoji.definition.as_ref().unwrap()
                ));
                return Ok(());
            }
        }
        return Ok(());
    }

    search_emojis(&emojis, search_term, num_results, show_number);
    Ok(())
}
