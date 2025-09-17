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
    pub fn new() -> Self {
        Self {
            model_path: PathBuf::from("data"),
        }
    }

    pub fn with_model(_model_id: String) -> Self {
        Self::new()
    }

    async fn download_model(&self) -> AnyhowResult<PathBuf> {
        // Create data directory if it doesn't exist
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

    async fn select_emoji_async(&self, situation: &str) -> Result<String> {
        // Download model if needed
        let model_path = self.download_model()
            .await
            .map_err(|e| EmoError::ConfigError(format!("Failed to download model: {}", e)))?;

        // Try to load and use the model - NO FALLBACKS
        self.run_inference(&model_path, situation)
    }

    fn run_inference(&self, model_path: &PathBuf, situation: &str) -> Result<String> {
        // Initialize parameters
        let params = LlamaParams::default();

        // Load the model
        let model = LlamaModel::load_from_file(model_path, params)
            .map_err(|e| EmoError::ConfigError(format!("Failed to load model: {}", e)))?;

        // Create prompt
        let prompt = format!(
            "You are an emoji selector. Reply with only a single emoji.\n\
            User: Select an emoji for: {}\n\
            Assistant:",
            situation
        );

        // Create session and feed prompt
        let mut session = model.create_session(SessionParams::default())
            .map_err(|e| EmoError::ConfigError(format!("Failed to create session: {}", e)))?;

        // Advance context with the prompt
        session.advance_context(&prompt)
            .map_err(|e| EmoError::ConfigError(format!("Failed to advance context: {}", e)))?;

        // Generate completion with greedy sampling for deterministic output
        let sampler = StandardSampler::new_greedy();

        let completions = session.start_completing_with(sampler, 10)
            .map_err(|e| EmoError::ConfigError(format!("Failed to start completion: {}", e)))?
            .into_strings();

        // Collect the output
        let mut output = String::new();
        for completion in completions {
            output.push_str(&completion);
            // Stop if we get enough characters
            if output.len() > 10 {
                break;
            }
        }

        // Extract emoji from output - propagate error if no emoji found
        self.extract_emoji(&output)
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