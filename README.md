# emo 🎯

A fast, command-line emoji search tool written in Rust with AI-powered selection. Find the perfect emoji for any occasion with intelligent search, AI context understanding, and custom mappings.

## Features

- 🔍 **Smart Search**: Search by emoji name, keywords, or description
- 🤖 **AI-Powered Selection**: Get contextually perfect emojis using LLM inference
- 📝 **Emoji Stories**: Generate emoji sentences that tell a story
- 💾 **Custom Mappings (Memos)**: Save frequently used emojis with personal shortcuts
- 🎲 **Random Emoji**: Get a random emoji for fun
- 📖 **Emoji Definitions**: Learn what emojis mean with detailed descriptions
- ⚡ **Lightning Fast**: Built in Rust for maximum performance
- 🎨 **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

### From Source

```bash
git clone https://github.com/yourusername/emo.git
cd emo
cargo build --release
sudo cp target/release/emo /usr/local/bin/
```

### From Release

Download the latest binary for your platform from the [releases page](https://github.com/yourusername/emo/releases).

## Usage

### Basic Search

Search for an emoji by name or keyword:

```bash
# Find a single emoji
emo fire
🔥

# Find multiple results
emo -c 3 happy
😊
😄
😃
```

### AI-Powered Selection (NEW in v2.1.0)

Let AI understand context and select the perfect emoji:

```bash
# Get contextually appropriate emoji
emo --ai "just fixed a bug"
🐛

# Multiple AI-selected emojis
emo --ai -c 3 "shipping to production"
🚀
📦
✨

# Generate an emoji story/sentence
emo --ai -s 5 "monday morning"
😴☕💼😅🏃

# Multiple emoji sentences
emo --ai -c 3 -s 4 "debugging"
🐛🔍💻🤔
😤🖥️❌😠
✅💡🎉👍
```

### Custom Mappings (Memos)

Save shortcuts for frequently used emojis:

```bash
# Save an emoji to a shortcut (now uses -m flag)
emo -m 🚀 deploy
deploy ➡ 🚀 ✅

# Use your shortcut
emo deploy
🚀

# With count: memo + search results
emo -c 3 deploy
🚀  # Your memo comes first
📦  # Then search results
🚢

# Save by search result index
emo -c 3 fire  # Shows 3 results
emo -m 2 flame  # Saves the 2nd result
```

### List and Manage Mappings

```bash
# List all saved mappings
emo -l
Saved mappings:
  deploy → 🚀
  fire → 🔥

# Erase a mapping
emo -e deploy
Mapping for 'deploy' erased ✅
```

### Other Features

```bash
# Get emoji definition
emo -d 🎉
🎉 - party popper A party popper, as blown at a celebration

# Get a random emoji
emo -r
🌈 - rainbow

# Show result numbers for easier selection
emo -n -c 5 heart
1. ❤️
2. 💜
3. 💙
4. 💚
5. 💛
```

## Command Reference

| Option | Description |
|--------|-------------|
| `-c, --count <N>` | Number of results to show (default: 1) |
| `-d, --define` | Show emoji definition |
| `-m, --memo <EMOJI>` | Save a mapping (replaces old -s) |
| `-s, --sentence <N>` | Generate emoji sentence of N length (AI mode) |
| `-e, --erase` | Remove a saved mapping |
| `-n, --number` | Display result numbers |
| `-l, --list-mappings` | List all saved mappings |
| `-r, --random` | Get a random emoji |
| `--ai` | Use AI for emoji selection |
| `--model <MODEL>` | Specify AI model (future use) |
| `-h, --help` | Show help information |

## Precedence Rules

When multiple selection methods are available, emo follows these priorities:

1. **Explicit mode flags** (highest priority)
   - `--ai` ignores memos and uses AI
   - `-r` always returns random
   - `-d` always shows definition

2. **Memos** (medium priority)
   - Normal search uses your saved mappings
   - With `-c`, memo appears first, then search results

3. **Search** (fallback)
   - Used when no memo exists

Example: If you memo "bug" → 🐞:
- `emo bug` → 🐞 (your memo)
- `emo --ai bug` → 🐛 (AI choice, ignores memo)
- `emo -c 3 bug` → 🐞 🐛 🦟 (memo + search)

## Configuration

Settings stored in your system's config directory:
- Linux: `~/.config/emo/config.json`
- macOS: `~/Library/Application Support/emo/config.json`
- Windows: `%APPDATA%\emo\config.json`

```json
{
  "mappings": {
    "deploy": "🚀",
    "fire": "🔥"
  },
  "model": null  // Optional: specify default AI model
}
```

## AI Models

The AI feature downloads a small language model (~1.6GB) on first use. Models are cached in:
- Linux: `~/.config/emo/models/`
- macOS: `~/Library/Application Support/emo/models/`

## Development

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test cli_integration  # End-to-end CLI tests
cargo test --test generators        # Unit tests
```

### Code Quality

```bash
cargo fmt        # Format code
cargo clippy     # Run linter
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Emoji data sourced from Unicode standards
- Built with [Rust](https://www.rust-lang.org/) and [clap](https://github.com/clap-rs/clap)
- AI powered by [llama.cpp](https://github.com/ggerganov/llama.cpp)