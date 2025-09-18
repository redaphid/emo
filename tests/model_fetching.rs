// Following ADD: Test dynamic model list fetching

use emo::models::ModelRegistry;

#[test]
fn model_registry_exists() {
    // Test 1: ModelRegistry type exists
    let _registry = ModelRegistry::new();
}

#[test]
fn model_registry_can_fetch() {
    // Test 2: ModelRegistry can fetch models
    let registry = ModelRegistry::new();
    let _result = registry.fetch_models();
}

#[test]
fn fetch_returns_models() {
    // Test 3: fetch_models returns actual models
    let registry = ModelRegistry::new();
    let models = registry.fetch_models().unwrap();

    // Should return at least one model
    assert!(!models.is_empty());
}

#[test]
fn fetched_models_have_valid_fields() {
    // Test 4: fetched models have required fields
    let registry = ModelRegistry::new();
    let models = registry.fetch_models().unwrap();

    if !models.is_empty() {
        let model = &models[0];
        assert!(!model.id.is_empty());
        assert!(!model.name.is_empty());
        assert!(model.url.starts_with("https://"));
        assert!(model.url.ends_with(".gguf"));
    }
}

#[test]
fn fetch_from_huggingface_api() {
    // Test 5: Actually fetch from HuggingFace API
    let registry = ModelRegistry::new();
    let models = registry.fetch_from_api().unwrap();

    // Should return multiple models
    assert!(models.len() > 1);
}

// TODO: Add test for real HTTP fetching when we implement ModelSource