use crate::router::catalog;
use crate::router::config;
use std::collections::HashMap;

pub fn select_model(scores: &HashMap<&str, f32>) -> (String, String, f32, f64, String) {
    // weights
    let weights = config::default_weights();

    let mut weighted = 0.0f32;
    let mut total_w = 0.0f32;
    for (k, &w) in weights.iter() {
        // `k` is `&'static str` (auto-deref handled), `w` is copied out
        let s = *scores.get(k).unwrap_or(&0.0);
        weighted += w * s;
        total_w += w;
    }

    let normalized = if total_w > 0.0 {
        weighted / total_w
    } else {
        weighted
    };

    // simple mapping
    let tier = if normalized > 0.85 {
        "REASONING"
    } else if normalized > 0.6 {
        "COMPLEX"
    } else if normalized > 0.3 {
        "MEDIUM"
    } else {
        "SIMPLE"
    };

    let map = config::tier_model_map();
    let model = map.get(tier).unwrap_or(&"openai/gpt-4o-mini").to_string();

    // cost estimate from catalog
    let pricing = catalog::default_pricing();
    let cost = *pricing.get(model.as_str()).unwrap_or(&1.0);

    let confidence = normalized;

    let explain = format!("weighted_score={:.3}", normalized);

    (model, tier.to_string(), confidence, cost, explain)
}
