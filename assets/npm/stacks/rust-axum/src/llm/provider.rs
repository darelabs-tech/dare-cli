//! LLM provider abstraction. Two implementations: Dummy and OpenAi.
//!
//! `async-trait` is used for dyn-compatible async traits — see Cargo.toml.
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
}

pub struct DummyProvider;

#[async_trait]
impl LlmProvider for DummyProvider {
    async fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String> {
        let n = prompt.len().min(max_tokens);
        Ok(format!("[dummy] {}", &prompt[..n]))
    }
}

pub struct OpenAiProvider {
    pub api_key: String,
    pub model: String,
    pub base_uri: String,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": max_tokens
        });
        let res = client
            .post(format!("{}/chat/completions", self.base_uri))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;
        let v: serde_json::Value = res.json().await?;
        Ok(v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }
}
