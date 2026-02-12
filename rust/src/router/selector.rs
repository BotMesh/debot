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

    // Tier thresholds tuned for lower cost bias:
    //   SIMPLE prompts:    0.00 – 0.12
    //   MEDIUM prompts:    0.12 – 0.26
    //   COMPLEX prompts:   0.26 – 0.36
    //   REASONING prompts: 0.36+
    let tier = if normalized > 0.36 {
        "REASONING"
    } else if normalized > 0.26 {
        "COMPLEX"
    } else if normalized > 0.12 {
        "MEDIUM"
    } else {
        "SIMPLE"
    };

    let map = config::available_tier_model_map();
    let model = map
        .get(tier)
        .copied()
        .unwrap_or_else(|| {
            // Fallback: try any available model from alternatives
            let alts = config::tier_alternatives();
            let alt_list = alts.get(tier).cloned().unwrap_or_default();
            let available = config::filter_available_models(alt_list);
            available.first().copied().unwrap_or("openai/gpt-4o-mini")
        })
        .to_string();

    // cost estimate from catalog
    let pricing = catalog::default_pricing();
    let cost = *pricing.get(model.as_str()).unwrap_or(&1.0);

    let confidence = normalized;

    let explain = format!("weighted_score={:.3}", normalized);

    (model, tier.to_string(), confidence, cost, explain)
}
