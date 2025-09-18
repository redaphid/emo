// Model definitions for AI emoji selection
use serde::{Deserialize, Serialize};
use crate::error::{Result, EmoError};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub size_mb: usize,
    pub description: String,
}

#[derive(Debug, Deserialize)]
struct HFModelResponse {
    #[serde(rename = "modelId")]
    model_id: String,
    downloads: Option<u64>,
    likes: Option<u64>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FileInfo {
    path: String,
    size: u64,
}

impl ModelInfo {
    pub fn filename(&self) -> String {
        // Extract filename from URL
        self.url.split('/').last().unwrap_or("model.gguf").to_string()
    }
}

pub struct ModelRegistry;

impl ModelRegistry {
    pub fn new() -> Self {
        ModelRegistry
    }

    pub fn fetch_models(&self) -> Result<Vec<ModelInfo>> {
        // Always fetch from remote, no fallbacks
        self.fetch_from_api()
    }

    pub fn fetch_from_api(&self) -> Result<Vec<ModelInfo>> {
        // NO FALLBACKS - fetch from remote or fail loudly
        self.fetch_remote_models()
    }

    fn fetch_remote_models(&self) -> Result<Vec<ModelInfo>> {
        // Fetch from HuggingFace API for small instruct GGUF models
        let url = "https://huggingface.co/api/models?search=GGUF+Q4_K_M&limit=10&sort=downloads";

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| EmoError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let response = client.get(url).send()
            .map_err(|e| EmoError::ConfigError(format!("Failed to fetch model list: {}", e)))?;

        if !response.status().is_success() {
            return Err(EmoError::ConfigError(format!(
                "Failed to fetch models: HTTP {}",
                response.status()
            )));
        }

        let hf_models: Vec<HFModelResponse> = response.json()
            .map_err(|e| EmoError::ConfigError(format!("Failed to parse model list: {}", e)))?;

        if hf_models.is_empty() {
            return Err(EmoError::ConfigError("No models found from HuggingFace".to_string()));
        }

        let mut models = Vec::new();

        // For each model, fetch its files to find the Q4_K_M GGUF file
        for hf_model in hf_models.iter().take(6) {
            // Fetch the model's file list
            let files_url = format!("https://huggingface.co/api/models/{}/tree/main", hf_model.model_id);

            if let Ok(files_response) = client.get(&files_url).send() {
                if files_response.status().is_success() {
                    // Parse files response
                    if let Ok(files_json) = files_response.text() {
                        // Find Q4_K_M GGUF file
                        if let Ok(files) = serde_json::from_str::<Vec<FileInfo>>(&files_json) {
                            for file in files {
                                if file.path.to_lowercase().contains("q4_k_m") &&
                                   file.path.ends_with(".gguf") {
                                    let size_mb = (file.size / 1_000_000) as usize;

                                    // Create size string
                                    let size_str = if size_mb < 1000 {
                                        format!("{}MB", size_mb)
                                    } else {
                                        format!("{:.1}GB", size_mb as f64 / 1000.0)
                                    };

                                    // Use the model ID as-is from HuggingFace
                                    // Clean it up slightly for display but don't hardcode names
                                    let repo_name = hf_model.model_id
                                        .split('/')
                                        .last()
                                        .unwrap_or(&hf_model.model_id);

                                    // Remove common suffixes for cleaner display
                                    let clean_name = repo_name
                                        .replace("-GGUF", "")
                                        .replace("-Q4_K_M", "")
                                        .replace("_", " ");

                                    // Create a short ID from the repo name
                                    // Take meaningful parts, limit length
                                    let id_parts: Vec<&str> = repo_name
                                        .split(|c: char| c == '-' || c == '_')
                                        .filter(|s| !s.is_empty() && s != &"GGUF" && s != &"Q4" && s != &"K" && s != &"M")
                                        .take(3)
                                        .collect();

                                    let id = id_parts.join("-").to_lowercase();

                                    models.push(ModelInfo {
                                        id,
                                        name: clean_name,
                                        url: format!("https://huggingface.co/{}/resolve/main/{}",
                                                   hf_model.model_id, file.path),
                                        size_mb,
                                        description: format!("Q4_K_M • {} • by {}",
                                            size_str,
                                            hf_model.model_id.split('/').next().unwrap_or("unknown")),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if models.is_empty() {
            return Err(EmoError::ConfigError("No compatible GGUF models found".to_string()));
        }

        Ok(models)
    }

}