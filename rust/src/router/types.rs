use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct RouteDecision {
    pub model: String,
    pub tier: String,
    pub confidence: f32,
    pub cost_estimate: f64,
    pub explain: String,
}
