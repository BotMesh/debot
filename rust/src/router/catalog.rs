use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
    pricing: Option<ModelPricing>,
}

#[derive(Deserialize)]
struct ModelPricing {
    prompt: Option<String>,
    completion: Option<String>,
}

pub fn default_pricing() -> HashMap<&'static str, f64> {
    static PRICING: OnceLock<HashMap<&'static str, f64>> = OnceLock::new();
    PRICING
        .get_or_init(|| {
            let mut m = HashMap::new();

            // Pull all models from OpenRouter so the catalog stays current.
            // OpenRouter's pricing fields are per-token USD; convert to USD per 1M output tokens.
            if let Ok(client) = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(6))
                .build()
            {
                if let Ok(resp) = client.get("https://openrouter.ai/api/v1/models").send() {
                    if let Ok(payload) = resp.json::<ModelsResponse>() {
                        for entry in payload.data {
                            if let Some(pricing) = entry.pricing {
                                if let Some(completion) = pricing.completion {
                                    if let Ok(price_per_token) = completion.parse::<f64>() {
                                        let key: &'static str =
                                            Box::leak(entry.id.into_boxed_str());
                                        m.insert(key, price_per_token * 1_000_000.0);
                                    }
                                } else if let Some(prompt) = pricing.prompt {
                                    if let Ok(price_per_token) = prompt.parse::<f64>() {
                                        let key: &'static str =
                                            Box::leak(entry.id.into_boxed_str());
                                        m.insert(key, price_per_token * 1_000_000.0);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Official provider pricing overrides (USD per 1M output tokens).
            // These keep core models aligned with vendor pricing instead of OpenRouter's pass-through.
            let overrides: [(&'static str, f64); 6] = [
                ("openai/gpt-3.5-turbo", 1.50),
                ("openai/gpt-4o-mini", 0.60),
                ("openai/o3", 8.00),
                ("anthropic/claude-opus-4-5", 25.00),
                ("deepseek/deepseek-chat", 0.42),
                ("minimax/minimax-m2", 1.20),
            ];
            for (model, price) in overrides {
                m.insert(model, price);
            }

            // README-referenced models to ensure a non-empty fallback when network is unavailable.
            m.entry("meta-llama/Llama-3.1-8B-Instruct").or_insert(0.0);
            m
        })
        .clone()
}
