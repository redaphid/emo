use clap::Parser;
use emo::{
    ai::AiEmojiSelector,
    error::{EmoError, Result},
    load_emojis, search, to_char, EmojiMappings, EmojiRecord,
};
use std::io::Write;

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
        help = "save a mapping for the search term to a specific emoji or index"
    )]
    save: Option<String>,
    #[arg(
        short = 'e',
        long,
        default_value_t = false,
        help = "erase the mapping for the specified search term"
    )]
    erase: bool,
    #[arg(short = 'n', long, help = "display the number of a given emoji result")]
    number: bool,
    #[arg(short = 'l', long, help = "list all saved mappings")]
    list_mappings: bool,
    #[arg(short = 'r', long, help = "get a random emoji")]
    random: bool,
    #[arg(long, help = "use AI to select the best emoji for your situation")]
    ai: bool,
    #[arg(long, help = "specify the AI model to use (default: gemma2:2b)")]
    model: Option<String>,
    #[arg(trailing_var_arg = true)]
    search_terms: Vec<String>,
}

fn print(results: &[(char, &EmojiRecord)], show_number: bool) {
    for (i, (emoji_char, _)) in results.iter().enumerate() {
        let prefix = if show_number {
            format!("{}. ", i + 1)
        } else {
            String::new()
        };

        try_print(&format!("{}{}", prefix, emoji_char));
    }
}

fn get_custom_emoji(search_term: &str) -> Result<Option<char>> {
    let mappings = EmojiMappings::load()?;
    Ok(mappings.mappings.get(search_term).cloned())
}

fn handle_search(search_term: &str, num_results: usize, show_number: bool) -> Result<()> {
    let emojis = load_emojis()?;

    // Get search results
    let mut results: Vec<(char, &EmojiRecord)> = Vec::new();

    // Check for custom emoji mapping first
    if let Some(custom_emoji) = get_custom_emoji(search_term)? {
        // Create a dummy EmojiRecord for custom mappings
        // We'll handle this specially in print
        if !results.is_empty() {
            return Ok(());
        }
        // For now, just print the custom emoji directly
        if show_number {
            try_print(&format!("1. {}", custom_emoji));
        } else {
            try_print(&format!("{}", custom_emoji));
        }
        return Ok(());
    }

    // Get regular search results
    results = search(&emojis, search_term, num_results);

    // Use the existing print function to display results
    print(&results, show_number);
    Ok(())
}

// Function to handle the define mode
fn handle_define(search_term: &str) -> Result<()> {
    if search_term.is_empty() {
        return Ok(());
    }

    let emojis = load_emojis()?;
    let first_char = search_term
        .chars()
        .next()
        .ok_or_else(|| EmoError::InvalidInput("Empty search term".to_string()))?;

    // Try direct lookup first
    for emoji in emojis {
        let emoji_char = match to_char(emoji) {
            Ok(ch) => ch,
            Err(_) => continue,
        };
        if emoji_char != first_char {
            continue;
        }
        let name = &emoji.name;
        let description = emoji.definition.as_deref().unwrap_or("");
        try_print(&format!("{} - {} {}", emoji_char, name, description));
        return Ok(());
    }

    // If exact emoji not found, fall back to search
    let results = search(&emojis, search_term, 1);
    if results.is_empty() {
        return Ok(());
    }

    let (emoji_char, emoji) = &results[0];
    let name = &emoji.name;
    let description = emoji.definition.as_deref().unwrap_or("");
    try_print(&format!("{} - {} {}", emoji_char, name, description));
    Ok(())
}

fn is_number(s: &str) -> bool {
    s.parse::<usize>().is_ok()
}

fn handle_save(emoji_to_save: &str, search_term: &str) -> Result<()> {
    if search_term.is_empty() || emoji_to_save.is_empty() {
        return Err(EmoError::InvalidInput(
            "Cannot save mapping for empty search term or emoji".to_string(),
        ));
    }

    let mut mappings = EmojiMappings::load()?;

    if is_number(emoji_to_save) {
        let index = emoji_to_save
            .parse::<usize>()
            .map_err(|_| EmoError::InvalidInput(format!("Invalid index: {}", emoji_to_save)))?;

        if index == 0 {
            return Err(EmoError::InvalidInput(
                "Index must be greater than 0".to_string(),
            ));
        }

        let emojis = load_emojis()?;
        let results = search(&emojis, search_term, index.max(1));

        if results.len() < index {
            return Err(EmoError::InvalidInput(format!(
                "Only {} results found, cannot select index {}",
                results.len(),
                index
            )));
        }

        let (emoji_char, _) = results[index - 1];

        mappings
            .mappings
            .insert(search_term.to_string(), emoji_char);
        mappings.save()?;

        try_print(&format!("{} ➡ {} ✅", search_term, emoji_char));
        return Ok(());
    }

    // Extract just the first character as the emoji
    let emoji_char = emoji_to_save
        .chars()
        .next()
        .ok_or_else(|| EmoError::InvalidInput("Empty emoji".to_string()))?;

    mappings
        .mappings
        .insert(search_term.to_string(), emoji_char);
    mappings.save()?;

    try_print(&format!("{} ➡ {} ✅", search_term, emoji_char));
    Ok(())
}

fn handle_erase(search_term: &str) -> Result<()> {
    if search_term.is_empty() {
        return Err(EmoError::InvalidInput(
            "Cannot erase mapping for empty search term".to_string(),
        ));
    }

    let mut mappings = EmojiMappings::load()?;
    if mappings.mappings.remove(search_term).is_none() {
        try_print(&format!("No mapping found for '{}'", search_term));
        return Ok(());
    }

    mappings.save()?;
    try_print(&format!("Mapping for '{}' erased ✅", search_term));
    Ok(())
}

fn main() {
    let result = run();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_list_mappings() -> Result<()> {
    let mappings = EmojiMappings::load()?;
    if mappings.mappings.is_empty() {
        try_print("No saved mappings.");
        return Ok(());
    }

    try_print("Saved mappings:");
    let mut entries: Vec<_> = mappings.mappings.iter().collect();
    entries.sort_by_key(|e| e.0);
    for (term, emoji) in entries {
        try_print(&format!("  {} → {}", term, emoji));
    }
    Ok(())
}

fn handle_random() -> Result<()> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let emojis = load_emojis()?;
    if emojis.is_empty() {
        return Err(EmoError::InvalidInput("No emojis available".to_string()));
    }

    // Use system time as a simple random seed
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = RandomState::new().build_hasher();
    seed.hash(&mut hasher);
    let index = (hasher.finish() as usize) % emojis.len();

    let emoji = &emojis[index];
    let emoji_char = to_char(emoji)?;
    try_print(&format!("{} - {}", emoji_char, emoji.name));
    Ok(())
}

fn handle_ai_emoji(situation: &str, model: Option<String>) -> Result<()> {
    let ai_selector = if let Some(model_name) = model {
        AiEmojiSelector::with_model(model_name)
    } else {
        AiEmojiSelector::new()
    };

    // Use the LLM for emoji selection - NO FALLBACKS, fail loudly
    let emoji = ai_selector.select_emoji_llm(situation)?;
    try_print(&emoji);
    Ok(())
}

fn run() -> Result<()> {
    let cmd = Cli::parse();

    // Handle list mappings
    if cmd.list_mappings {
        return handle_list_mappings();
    }

    // Handle random emoji
    if cmd.random {
        return handle_random();
    }

    if cmd.search_terms.is_empty() {
        return Err(EmoError::InvalidInput(
            "Please provide a search term or situation".to_string(),
        ));
    }

    let search_term = &cmd.search_terms.join(" ");
    let num_results = cmd.count;
    let define = cmd.define;
    let show_number = cmd.number;

    // Handle AI mode
    if cmd.ai {
        return handle_ai_emoji(search_term, cmd.model);
    }

    // Handle erasing a mapping first
    if cmd.erase {
        return handle_erase(search_term);
    }

    // Handle saving a mapping
    if let Some(emoji_to_save) = &cmd.save {
        // The command format is: emo -s EMOJI SEARCH_TERM
        // So emoji_to_save is the emoji and search_term is what we want to map it to
        return handle_save(emoji_to_save, search_term);
    }

    // Handle define mode
    if define {
        return handle_define(search_term);
    }

    // Handle search mode (which now handles custom mappings internally)
    handle_search(search_term, num_results, show_number)
}
