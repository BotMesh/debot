use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

/// Global set of available provider names (set by Python at startup).
/// If `None`, all models are considered available (backward compat).
static AVAILABLE_PROVIDERS: OnceLock<Mutex<Option<HashSet<String>>>> = OnceLock::new();

fn providers_lock() -> &'static Mutex<Option<HashSet<String>>> {
    AVAILABLE_PROVIDERS.get_or_init(|| Mutex::new(None))
}

/// Maps config provider name → model prefixes it can serve.
fn provider_prefixes() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert("anthropic", vec!["anthropic/"]);
    m.insert("openai", vec!["openai/"]);
    m.insert("groq", vec!["groq/"]);
    m.insert("nvidia", vec!["moonshotai/"]);
    m.insert("moonshotai", vec!["moonshotai/"]);
    m.insert("deepseek", vec!["deepseek/"]);
    m.insert("gemini", vec!["gemini/", "google/"]);
    m.insert("zhipu", vec!["zhipu/"]);
    // openrouter is a meta-provider: all models pass
    m.insert("openrouter", vec![]);
    m.insert("minimax", vec!["minimax/"]);
    m
}

/// Set the available providers (called from Python at startup).
pub fn set_available_providers(providers: Vec<String>) {
    let mut guard = providers_lock().lock().unwrap();
    *guard = Some(providers.into_iter().collect());
}

/// Reset available providers to None (all models available). For testing.
pub fn reset_available_providers() {
    let mut guard = providers_lock().lock().unwrap();
    *guard = None;
}

/// Check if a model is available given the configured providers.
/// Returns true if no providers are configured (backward compat).
pub fn is_model_available(model: &str) -> bool {
    let guard = providers_lock().lock().unwrap();
    let providers = match guard.as_ref() {
        Some(p) => p,
        None => return true, // no filtering configured
    };

    // If openrouter is configured, all models are available
    if providers.contains("openrouter") {
        return true;
    }

    let prefixes = provider_prefixes();
    for provider in providers {
        if let Some(pfxs) = prefixes.get(provider.as_str()) {
            for pfx in pfxs {
                if model.starts_with(pfx) {
                    return true;
                }
            }
        }
    }
    false
}

/// Filter a list of models to only those available.
pub fn filter_available_models(models: Vec<&str>) -> Vec<&str> {
    models
        .into_iter()
        .filter(|m| is_model_available(m))
        .collect()
}

/// Like `tier_model_map()` but if the default model for a tier is unavailable,
/// picks the first available alternative for that tier.
pub fn available_tier_model_map() -> HashMap<&'static str, &'static str> {
    let defaults = tier_model_map();
    let alts = tier_alternatives();

    let mut result = HashMap::new();
    for (&tier, &default_model) in &defaults {
        if is_model_available(default_model) {
            result.insert(tier, default_model);
        } else {
            // Try alternatives for this tier
            let alt_list = alts.get(tier).cloned().unwrap_or_default();
            let available = filter_available_models(alt_list);
            if let Some(&first) = available.first() {
                result.insert(tier, first);
            } else {
                // No available model found; keep default as last resort
                result.insert(tier, default_model);
            }
        }
    }
    result
}

pub fn default_weights() -> HashMap<&'static str, f32> {
    let mut m = HashMap::new();
    m.insert("reasoning", 0.22);
    m.insert("code", 0.18);
    m.insert("multistep", 0.15);
    m.insert("technical", 0.12);
    m.insert("token_count", 0.10);
    m.insert("creative", 0.06);
    m.insert("question", 0.05);
    m.insert("imperative", 0.04);
    m.insert("format", 0.04);
    m.insert("negation", 0.04);
    m
}

pub fn tier_model_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("SIMPLE", "openai/gpt-3.5-turbo");
    m.insert("MEDIUM", "openai/gpt-4o-mini");
    m.insert("COMPLEX", "anthropic/claude-opus-4-5");
    m.insert("REASONING", "openai/o3");
    m
}

/// Ordered tier list from lowest to highest complexity.
pub const TIER_ORDER: [&str; 4] = ["SIMPLE", "MEDIUM", "COMPLEX", "REASONING"];

/// Returns the next higher tier for escalation, or None if already at top.
pub fn next_tier(current: &str) -> Option<&'static str> {
    let idx = TIER_ORDER.iter().position(|t| *t == current)?;
    TIER_ORDER.get(idx + 1).copied()
}

/// Alternative models per tier, sorted by cost ascending (cheapest first).
/// Includes models from multiple providers for cross-provider billing fallback.
pub fn tier_alternatives() -> HashMap<&'static str, Vec<&'static str>> {
    let mut m = HashMap::new();
    m.insert(
        "SIMPLE",
        vec![
            "groq/llama-3.3-70b-versatile", // free tier
            "deepseek/deepseek-chat",       // $0.42
            "openai/gpt-4o-mini",           // $0.60
            "moonshotai/kimi-k2.5",         // $1.40 (nvidia)
            "openai/gpt-3.5-turbo",         // $1.50
            "anthropic/claude-haiku-3-5",   // $5.00
        ],
    );
    m.insert(
        "MEDIUM",
        vec![
            "groq/llama-3.3-70b-versatile", // free tier
            "deepseek/deepseek-chat",       // $0.42
            "openai/gpt-4o-mini",           // $0.60
            "minimax/minimax-m2",           // $1.20
            "moonshotai/kimi-k2.5",         // $1.40 (nvidia)
            "anthropic/claude-haiku-3-5",   // $5.00
        ],
    );
    m.insert(
        "COMPLEX",
        vec![
            "groq/llama-3.3-70b-versatile", // free tier (best-effort)
            "moonshotai/kimi-k2.5",         // $1.40 (nvidia)
            "openai/gpt-4o",                // $10.00
            "anthropic/claude-sonnet-4-5",  // $15.00
            "anthropic/claude-opus-4-5",    // $25.00
        ],
    );
    m.insert(
        "REASONING",
        vec![
            "groq/llama-3.3-70b-versatile", // free tier (best-effort)
            "deepseek/deepseek-reasoner",   // $2.19
            "openai/o3-mini",               // $4.40
            "openai/o3",                    // $8.00
            "anthropic/claude-opus-4-5",    // $25.00 (can reason)
        ],
    );
    m
}
