# Claude Development Notes for emo

## Project Overview
`emo` is a CLI tool for finding and managing emojis with AI-powered selection capabilities.

## Recent Changes (v2.1.0)

### Command Line Interface Updates
- **Changed**: `-s` flag is now `--sentence` (generates emoji sentences)
- **Changed**: `-m/--memo` replaces old `-s` (saves emoji mappings)
- **Both -c and -s together**: Generates multiple sentences (e.g., `-c 3 -s 5` = 3 sentences of 5 emojis each)

### Precedence Rules (IMPORTANT)
When multiple modes are available, follow this precedence:

1. **Explicit mode flags (highest priority)**:
   - `--ai` → Always use AI, ignores memos
   - `-r/--random` → Always random
   - `-d/--define` → Always show definition

2. **Memoization (medium priority)**:
   - Used for default `emo <term>` searches
   - When using `-c` with memo: memo comes FIRST, then (count-1) search results
   - Example: `emo -m "🚀" deploy` then `emo -c 3 deploy` → 🚀, then 2 other deploy-related emojis

3. **Search (lowest priority)**:
   - Fallback when no memo exists

### Configuration
- Config file location: `~/Library/Application Support/emo/config.json` (macOS) or `~/.config/emo/config.json` (Linux)
- Structure:
```json
{
  "mappings": {
    "deploy": "🚀",
    "fire": "🔥"
  },
  "model": "phi-2:q4"  // Optional, defaults to null
}
```

### AI Model
- Downloads to: `~/Library/Application Support/emo/models/` (macOS)
- Model: Phi-2 Q4_K_M quantized (~1.6GB)
- URL: `https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q4_K_M.gguf`
- Temperature: Using `StandardSampler::new_softmax(vec![], 2)` for focused but varied output

### Testing Strategy
- Unit tests: `tests/generators.rs` - Test each generator independently
- CLI integration: `tests/cli_integration.rs` - Full end-to-end tests
- Property tests: `tests/property_tests.rs` - Invariants

### Current Architecture Issues & TODOs

#### CRITICAL BUG TO FIX
The memo + count behavior (`emo -c 3 deploy` with memo) is not working correctly. The code in `handle_search` needs fixing - it's only returning 1 result instead of 3.

#### Refactoring Needed
Current code has too much flag-specific branching. Need to implement:

1. **Generator trait system**:
```rust
trait EmojiGenerator {
    fn generate(&self, input: &str) -> Result<String>;
}
```

2. **Generators to implement**:
   - `SearchGenerator` - searches emoji database
   - `AiGenerator` - uses LLM for single emoji
   - `AiSentenceGenerator` - generates emoji sentences
   - `RandomGenerator` - random emoji
   - `MemoGenerator` - returns saved mappings
   - `CompositeGenerator` - memo + search for `-c`

3. **Clean separation**: The count logic should be OUTSIDE generators, just calling them N times

#### Model Selection
- `with_model()` currently ignores the model parameter
- Download is hardcoded to phi-2
- Need model registry for future expansion

### Development Methodology
Following ADD (Asshole Driven Development):
- Write minimal failing test first
- Implement ONLY enough to pass
- No anticipating future needs
- Force generalization through counter-examples

### Key Functions

- `handle_search()`: Main search logic, handles memos and count
- `handle_ai_emoji()`: AI emoji generation with deduplication
- `handle_ai_sentence()`: Emoji sentence generation
- `handle_save()`: Save memo mappings
- `AiEmojiSelector::select_emoji_with_exclusions()`: Ensures unique emojis with `-c`

### Dependencies
- `clap` - CLI parsing
- `llama_cpp` - LLM inference
- `dirs` - Platform-specific directories
- `serde/serde_json` - Config serialization
- Test deps: `assert_cmd`, `predicates`, `tempfile`, `proptest`

## Next Steps
1. Fix memo + count bug in `handle_search`
2. Complete generator trait refactoring
3. Add default config bundling
4. Implement proper model selection
5. Add comprehensive test coverage