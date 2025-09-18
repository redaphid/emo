use crate::error::{EmoError, Result};
use anyhow::Result as AnyhowResult;
use llama_cpp::{
    LlamaModel, LlamaParams, SessionParams,
    standard_sampler::StandardSampler,
};
use std::io::Write;
use std::path::PathBuf;
use tokio::runtime::Runtime;

pub struct AiEmojiSelector {
    model_path: PathBuf,
}

impl AiEmojiSelector {
    fn get_config_dir() -> PathBuf {
        // Check for XDG_CONFIG_HOME first (for testing and custom configs)
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg_config);
        }

        // Fall back to system default
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn new() -> Self {
        let model_path = Self::get_config_dir()
            .join("emo")
            .join("models");

        Self { model_path }
    }

    pub fn with_model(_model_id: String) -> Self {
        Self::new()
    }

    async fn download_model(&self) -> AnyhowResult<PathBuf> {
        // Create models directory in ~/.config/emo/models if it doesn't exist
        std::fs::create_dir_all(&self.model_path)?;

        let model_file = self.model_path.join("phi-2-q4.gguf");

        // Check if model already exists
        if model_file.exists() {
            return Ok(model_file);
        }

        eprintln!("Downloading Phi-2 model (this is a one-time download of ~1.6GB)...");
        eprintln!("This will enable AI-powered emoji selection.");

        // Download from HuggingFace - Phi-2 is known to work with llama.cpp
        let url = "https://huggingface.co/TheBloke/phi-2-GGUF/resolve/main/phi-2.Q4_K_M.gguf";

        let response = reqwest::blocking::get(url)?;
        let mut file = std::fs::File::create(&model_file)?;

        let content = response.bytes()?;
        file.write_all(&content)?;

        eprintln!("Model downloaded successfully!");
        Ok(model_file)
    }

    pub fn select_emoji_llm(&self, situation: &str) -> Result<String> {
        // Use tokio runtime to handle async operations
        let rt = Runtime::new().map_err(|e| {
            EmoError::ConfigError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            self.select_emoji_async(situation).await
        })
    }

    pub fn select_emoji_with_exclusions(&self, situation: &str, exclude: &[String]) -> Result<String> {
        // Use tokio runtime to handle async operations
        let rt = Runtime::new().map_err(|e| {
            EmoError::ConfigError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            self.select_emoji_with_exclusions_async(situation, exclude).await
        })
    }


    pub fn generate_emoji_sentence(&self, situation: &str, length: usize) -> Result<String> {
        // Use tokio runtime to handle async operations
        let rt = Runtime::new().map_err(|e| {
            EmoError::ConfigError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            self.generate_sentence_async(situation, length).await
        })
    }

    async fn select_emoji_async(&self, situation: &str) -> Result<String> {
        // Download model if needed
        let model_path = self.download_model()
            .await
            .map_err(|e| EmoError::ConfigError(format!("Failed to download model: {}", e)))?;

        // Try to load and use the model - NO FALLBACKS
        self.run_inference(&model_path, situation, &[])
    }

    async fn select_emoji_with_exclusions_async(&self, situation: &str, exclude: &[String]) -> Result<String> {
        // Download model if needed
        let model_path = self.download_model()
            .await
            .map_err(|e| EmoError::ConfigError(format!("Failed to download model: {}", e)))?;

        // Try to load and use the model with exclusions
        self.run_inference(&model_path, situation, exclude)
    }


    async fn generate_sentence_async(&self, situation: &str, length: usize) -> Result<String> {
        // Download model if needed
        let model_path = self.download_model()
            .await
            .map_err(|e| EmoError::ConfigError(format!("Failed to download model: {}", e)))?;

        // Generate an emoji sentence
        self.run_sentence_inference(&model_path, situation, length)
    }

    fn run_inference(&self, model_path: &PathBuf, situation: &str, exclude: &[String]) -> Result<String> {
        // Initialize parameters
        let params = LlamaParams::default();

        // Load the model
        let model = LlamaModel::load_from_file(model_path, params)
            .map_err(|e| EmoError::ConfigError(format!("Failed to load model: {}", e)))?;

        // Create a stronger prompt that encourages emoji-only output
        let prompt = if exclude.is_empty() {
            format!(
                "Task: Select ONE emoji.\nSituation: {}\nEmoji:",
                situation
            )
        } else {
            let exclusions = exclude.join(", ");
            format!(
                "Task: Select ONE emoji.\nSituation: {}\nAlready used emojis (do not repeat these): {}\nEmoji:",
                situation, exclusions
            )
        };

        // Create session and feed prompt
        let mut session = model.create_session(SessionParams::default())
            .map_err(|e| EmoError::ConfigError(format!("Failed to create session: {}", e)))?;

        // Advance context with the prompt
        session.advance_context(&prompt)
            .map_err(|e| EmoError::ConfigError(format!("Failed to advance context: {}", e)))?;

        // Use softmax with fewer candidates for more focused selection (lower temperature)
        let sampler = StandardSampler::new_softmax(vec![], 2);

        let completions = session.start_completing_with(sampler, 5)  // Just a few tokens for emoji
            .map_err(|e| EmoError::ConfigError(format!("Failed to start completion: {}", e)))?
            .into_strings();

        // Collect and immediately check for emoji in each token
        for completion in completions {
            // Check if this completion has an emoji
            if let Ok(emoji) = self.extract_emoji(&completion) {
                return Ok(emoji);
            }
        }

        // No emoji found, fail loudly
        Err(EmoError::ConfigError("LLM did not generate an emoji".to_string()))
    }

    fn extract_emoji(&self, output: &str) -> Result<String> {
        // Try to find an emoji character in the output
        for ch in output.chars() {
            if is_emoji_char(ch) {
                return Ok(ch.to_string());
            }
        }
        // NO FALLBACK - fail loudly if no emoji found
        Err(EmoError::ConfigError("No emoji found in LLM output".to_string()))
    }


    fn run_sentence_inference(&self, model_path: &PathBuf, situation: &str, length: usize) -> Result<String> {
        // Initialize parameters
        let params = LlamaParams::default();

        // Load the model
        let model = LlamaModel::load_from_file(model_path, params)
            .map_err(|e| EmoError::ConfigError(format!("Failed to load model: {}", e)))?;

        // Create prompt for emoji sentence
        let prompt = format!(
            "You are an emoji generator. Create a sequence of exactly {} emojis that tell a story about: {}\n\
            Reply with ONLY emojis, no text.\n\
            Assistant:",
            length, situation
        );

        // Create session and feed prompt
        let mut session = model.create_session(SessionParams::default())
            .map_err(|e| EmoError::ConfigError(format!("Failed to create session: {}", e)))?;

        // Advance context with the prompt
        session.advance_context(&prompt)
            .map_err(|e| EmoError::ConfigError(format!("Failed to advance context: {}", e)))?;

        // Use softmax with fewer candidates for more focused selection (lower temperature)
        let sampler = StandardSampler::new_softmax(vec![], 2);

        let completions = session.start_completing_with(sampler, 50)  // More tokens for sentence
            .map_err(|e| EmoError::ConfigError(format!("Failed to start completion: {}", e)))?
            .into_strings();

        // Collect the output
        let mut output = String::new();
        let mut emoji_count = 0;

        for completion in completions {
            for ch in completion.chars() {
                if is_emoji_char(ch) {
                    output.push(ch);
                    emoji_count += 1;
                    if emoji_count >= length {
                        return Ok(output);
                    }
                }
            }
        }

        // If we didn't get enough emojis, fail loudly
        if emoji_count == 0 {
            Err(EmoError::ConfigError("No emojis generated for sentence".to_string()))
        } else {
            Ok(output)  // Return what we got even if less than requested
        }
    }
}

fn is_emoji_char(ch: char) -> bool {
    // Check if character is in emoji ranges
    matches!(ch as u32,
        0x1F300..=0x1F9FF | // Emoticons & misc
        0x2600..=0x26FF |   // Misc symbols
        0x2700..=0x27BF |   // Dingbats
        0x1F000..=0x1F02F | // Mahjong/Domino
        0x1FA70..=0x1FAFF   // More symbols
    )
}