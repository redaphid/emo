use crate::error::{EmoError, Result};
use anyhow::Result as AnyhowResult;
use hf_hub::api::sync::ApiBuilder;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::num::NonZeroU32;
use std::path::PathBuf;

pub struct AiEmojiSelector {
    model_path: PathBuf,
    model_override: Option<String>,
    backend: LlamaBackend,
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
        Self::with_override(None)
    }

    pub fn with_model(model_id: String) -> Self {
        Self::with_override(Some(model_id))
    }

    fn with_override(model_override: Option<String>) -> Self {
        let mut backend = LlamaBackend::init()
            .unwrap_or_else(|e| panic!("Failed to init backend: {}", e));
        backend.void_logs();

        Self {
            model_path: Self::get_config_dir().join("emo").join("models"),
            backend,
            model_override,
        }
    }

    fn download_model_sync(&self) -> AnyhowResult<PathBuf> {
        // Create models directory in ~/.config/emo/models if it doesn't exist
        std::fs::create_dir_all(&self.model_path)?;

        // Use override if provided, otherwise check config
        let model_id = self.model_override.clone()
            .or_else(|| crate::EmojiMappings::load().ok()?.model);

        // Fetch available models from registry
        let registry = crate::models::ModelRegistry::new();
        let available_models = registry.fetch_models()
            .map_err(|e| anyhow::anyhow!("Failed to fetch models: {}", e))?;

        if available_models.is_empty() {
            return Err(anyhow::anyhow!("No models available from HuggingFace"));
        }

        // Find the model by ID or use the first available
        let model = match model_id {
            Some(id) => available_models.iter()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("Model '{}' not found", id))?,
            None => &available_models[0],
        };

        let url = &model.url;
        let filename = model.filename();

        let model_file = self.model_path.join(filename);

        // Check if model already exists (silently return if it does)
        if model_file.exists() {
            return Ok(model_file);
        }

        // Try HuggingFace Hub API first
        if let Ok(api) = ApiBuilder::new().with_progress(true).build() {
            // Parse the URL to get repo info
            let url_parts: Vec<&str> = model.url.split('/').collect();
            if url_parts.len() > 5 {
                let owner = url_parts[3];
                let repo_name = url_parts[4];
                let repo_id = format!("{}/{}", owner, repo_name);

                // The HF API will use cached version if available, only print messages if actually downloading
                // We can't easily detect if it's cached beforehand, so we'll rely on the progress bar
                if let Ok(path) = api.model(repo_id).get(&model.filename()) {
                    // Model was either cached or downloaded - the progress bar handles the feedback
                    return Ok(path);
                }
            }
        }

        // Fallback to direct download
        eprintln!("📥 Downloading {} model ({})...", model.name, model.id);
        eprintln!("This is a one-time download for AI-powered emoji selection.");

        // Create progress bar
        let client = reqwest::blocking::Client::new();
        let mut response = client.get(url).send()?;

        let total_size = response
            .content_length()
            .unwrap_or(1_600_000_000); // Default to ~1.6GB if unknown

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Failed to set progress bar template")
                .progress_chars("#>-")
        );

        // Download with progress
        let mut file = std::fs::File::create(&model_file)?;
        let mut downloaded = 0u64;
        let mut buffer = [0; 8192];

        loop {
            use std::io::Read;
            let bytes_read = response.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            file.write_all(&buffer[..bytes_read])?;
            downloaded += bytes_read as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("✅ Model downloaded successfully!");
        Ok(model_file)
    }

    pub fn select_emoji_llm(&self, situation: &str) -> Result<String> {
        self.select_emoji_with_exclusions(situation, &[])
    }

    pub fn select_emoji_with_exclusions(&self, situation: &str, exclude: &[String]) -> Result<String> {
        // Download model if needed
        let model_path = self.download_model_sync()
            .map_err(|e| EmoError::ConfigError(format!("Failed to download model: {}", e)))?;

        // Set up model parameters
        let model_params = LlamaModelParams::default();

        // Load the model
        let model = LlamaModel::load_from_file(&self.backend, model_path, &model_params)
            .map_err(|e| EmoError::ConfigError(format!("Failed to load model: {}", e)))?;

        // Create context parameters
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(2048).unwrap()));

        // Create context
        let mut ctx = model.new_context(&self.backend, ctx_params)
            .map_err(|e| EmoError::ConfigError(format!("Failed to create context: {}", e)))?;

        // Create a prompt that encourages emoji-only output
        let prompt = match exclude.is_empty() {
            true => format!("Task: Select ONE emoji that best represents: {}. Reply with only the emoji, nothing else.\nEmoji:", situation),
            false => format!("Task: Select ONE emoji that best represents: {}. Do not use: {}. Reply with only the emoji.\nEmoji:", situation, exclude.join(", ")),
        };

        // Tokenize the prompt
        let tokens_list = model.str_to_token(&prompt, AddBos::Always)
            .map_err(|e| EmoError::ConfigError(format!("Failed to tokenize: {}", e)))?;

        // Create batch for prompt processing
        let mut batch = LlamaBatch::new(512, 1);

        let last_index = tokens_list.len() - 1;
        for (i, token) in tokens_list.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == last_index)
                .map_err(|e| EmoError::ConfigError(format!("Failed to add to batch: {}", e)))?;
        }

        // Process the prompt
        ctx.decode(&mut batch)
            .map_err(|e| EmoError::ConfigError(format!("Failed to decode: {}", e)))?;

        // Generate tokens looking for an emoji
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(0.2),
            LlamaSampler::dist(1234),
        ]);

        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut n_cur = batch.n_tokens();

        // Generate up to 20 tokens
        for _ in 0..20 {
            // Sample next token (always from the last position in the batch)
            let new_token_id = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(new_token_id);

            // Check if it's EOG (end of generation)
            if model.is_eog_token(new_token_id) {
                break;
            }

            // Decode token to string
            let token_bytes = model.token_to_bytes(new_token_id, Special::Tokenize)
                .map_err(|e| EmoError::ConfigError(format!("Failed to get token bytes: {}", e)))?;

            let mut token_str = String::with_capacity(32);
            let (_result, _read, _had_errors) = decoder.decode_to_string(&token_bytes, &mut token_str, false);

            output.push_str(&token_str);

            // Check if we found an emoji
            for ch in token_str.chars() {
                if is_emoji_char(ch) {
                    return Ok(ch.to_string());
                }
            }

            // Add token to batch for next iteration
            batch.clear();
            batch.add(new_token_id, n_cur, &[0], true)
                .map_err(|e| EmoError::ConfigError(format!("Failed to add to batch: {}", e)))?;

            n_cur += 1;

            ctx.decode(&mut batch)
                .map_err(|e| EmoError::ConfigError(format!("Failed to decode: {}", e)))?;

            // Stop if we generated enough text
            if output.len() > 50 {
                break;
            }
        }

        // No emoji found - fail loudly
        Err(EmoError::ConfigError(format!(
            "LLM did not generate an emoji. Generated text: '{}'",
            output
        )))
    }

    pub fn generate_emoji_sentence(&self, situation: &str, length: usize) -> Result<String> {
        let mut emojis = Vec::new();
        let mut exclude = Vec::new();

        for _ in 0..length {
            let emoji = self.select_emoji_with_exclusions(situation, &exclude)?;
            emojis.push(emoji.clone());
            exclude.push(emoji);
        }

        Ok(emojis.join(""))
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