use pyo3::prelude::*;
use serde_json::json;

use crate::router::scorer;
use crate::router::selector;

#[pyfunction]
fn route_text(prompt: &str, _max_tokens: usize) -> PyResult<String> {
    let scores = scorer::score_text(prompt);
    let (model, tier, confidence, cost, explain) = selector::select_model(&scores);

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

pub fn pybindings(m: &pyo3::Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(route_text, m)?)?;
    Ok(())
}

// (no re-exports here) router exposes `pybindings` which is called from lib.rs
