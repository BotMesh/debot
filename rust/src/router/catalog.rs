use std::collections::HashMap;

pub fn default_pricing() -> HashMap<&'static str, f64> {
    let mut m = HashMap::new();
    m.insert("deepseek/deepseek-chat", 0.27);
    m.insert("openai/gpt-4o-mini", 0.60);
    m.insert("anthropic/claude-sonnet-4", 15.0);
    m.insert("openai/o3", 10.0);
    m
}
