# emo 🎯

A fast, command-line emoji search tool written in Rust. Find the perfect emoji for any occasion with intelligent search and custom mappings.

## Features

- 🔍 **Smart Search**: Search by emoji name, keywords, or description
- 💾 **Custom Mappings**: Save frequently used emojis with personal shortcuts
- 🎲 **Random Emoji**: Get a random emoji for fun
- 📝 **Emoji Definitions**: Learn what emojis mean with detailed descriptions
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
emo smile
😊

# Find multiple results
emo -c 5 smile
😊
😄
😃
😀
😁
```

### Custom Mappings

Save shortcuts for frequently used emojis:

```bash
# Save an emoji to a shortcut
emo -s 🎉 party
party ➡ 🎉 ✅

# Use your shortcut
emo party
🎉

# Save by search result index
emo -c 3 fire  # Shows 3 results
emo -s 2 flame  # Saves the 2nd result
flame ➡ 🔥 ✅
```

### List and Manage Mappings

```bash
# List all saved mappings
emo -l
Saved mappings:
  flame → 🔥
  party → 🎉

# Erase a mapping
emo -e party
Mapping for 'party' erased ✅
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
| `-s, --save <EMOJI>` | Save a mapping |
| `-e, --erase` | Remove a saved mapping |
| `-n, --number` | Display result numbers |
| `-l, --list-mappings` | List all saved mappings |
| `-r, --random` | Get a random emoji |
| `-h, --help` | Show help information |

## Configuration

Custom mappings are stored in your system's config directory:
- Linux: `~/.config/emo/mappings.json`
- macOS: `~/Library/Application Support/emo/mappings.json`
- Windows: `%APPDATA%\emo\mappings.json`

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
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