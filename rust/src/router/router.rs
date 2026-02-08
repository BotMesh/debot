use pyo3::prelude::*;
use serde_json::json;

use crate::router::catalog;
use crate::router::config;
use crate::router::metrics;
use crate::router::scorer;
use crate::router::selector;

#[pyfunction]
fn route_text(prompt: &str, _max_tokens: usize) -> PyResult<String> {
    let scores = scorer::score_text(prompt);
    let (model, tier, confidence, cost, explain) = selector::select_model(&scores);
    metrics::record_decision(&model, &tier, confidence, cost);

    let decision = json!({
        "model": model,
        "tier": tier,
        "confidence": confidence,
        "cost_estimate": cost,
        "explain": explain,
        "scores": scores,
    });

    Ok(decision.to_string())
}

/// Returns the context window size (max tokens) for a model, or 0 if unknown.
#[pyfunction]
fn get_context_length(model: &str) -> PyResult<u64> {
    let ctx_map = catalog::default_context_lengths();
    Ok(*ctx_map.get(model).unwrap_or(&0))
}

/// Returns a JSON object with the next tier's model for escalation, or empty string if at top.
/// JSON: {"model": "...", "tier": "...", "cost": ...}
#[pyfunction]
fn get_fallback_model(current_tier: &str) -> PyResult<String> {
    let next = config::next_tier(current_tier);
    match next {
        Some(next_tier) => {
            let map = config::tier_model_map();
            let model = map.get(next_tier).unwrap_or(&"openai/gpt-4o-mini");
            let pricing = catalog::default_pricing();
            let cost = *pricing.get(model).unwrap_or(&1.0);
            let result = json!({
                "model": model,
                "tier": next_tier,
                "cost": cost,
            });
            Ok(result.to_string())
        }
        None => Ok(String::new()),
    }
}

/// Returns a JSON array of alternative models for a tier, sorted by cost ascending.
/// Each entry: {"model": "...", "cost": ...}
/// Used for billing fallback: try same-tier alternatives before escalating.
#[pyfunction]
fn get_tier_alternatives(tier: &str) -> PyResult<String> {
    let alts = config::tier_alternatives();
    let pricing = catalog::default_pricing();
    let models = alts.get(tier).cloned().unwrap_or_default();
    let mut entries: Vec<serde_json::Value> = models
        .iter()
        .map(|&model| {
            let cost = *pricing.get(model).unwrap_or(&1.0);
            json!({"model": model, "cost": cost})
        })
        .collect();
    // Sort by cost ascending (cheapest first)
    entries.sort_by(|a, b| {
        let ca = a["cost"].as_f64().unwrap_or(f64::MAX);
        let cb = b["cost"].as_f64().unwrap_or(f64::MAX);
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()))
}

pub fn pybindings(m: &pyo3::Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(route_text, m)?)?;
    m.add_function(wrap_pyfunction!(get_context_length, m)?)?;
    m.add_function(wrap_pyfunction!(get_fallback_model, m)?)?;
    m.add_function(wrap_pyfunction!(get_tier_alternatives, m)?)?;
    m.add_function(wrap_pyfunction!(metrics::get_router_metrics, m)?)?;
    m.add_function(wrap_pyfunction!(metrics::reset_router_metrics, m)?)?;
    m.add_function(wrap_pyfunction!(metrics::get_router_metrics_count, m)?)?;
    m.add_function(wrap_pyfunction!(metrics::record_escalation, m)?)?;
    Ok(())
}

// (no re-exports here) router exposes `pybindings` which is called from lib.rs
