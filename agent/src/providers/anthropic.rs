use crate::providers::LLMProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use crate::types::{ChatMessage, ToolDefinition, LLMResponse};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider { // Constructor
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    /// Create provider with Claude Sonnet (recommended)
    #[allow(unused)]
    pub fn with_claude_sonnet(api_key: &str) -> Self {
        Self::new(api_key, "claude-3-5-sonnet-20241022")
    }

    /// Create provider with Claude Haiku (fastest)
    #[allow(unused)]
    pub fn with_claude_haiku(api_key: &str) -> Self {
        Self::new(api_key, "claude-3-5-haiku-20241022")
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn generate_response(
        &self,
        messages: &[ChatMessage],
        _tools: &[ToolDefinition],
    ) -> Result<LLMResponse, Box<dyn Error>> {
        // Convert messages to Anthropic format
        let anthropic_messages: Vec<Value> = messages.iter()
            .map(|msg| convert_to_anthropic_message(msg))
            .collect();

        // Create request
        let request = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": anthropic_messages
        });

        // Make API call
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        // Parse response
        let response_json: Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LLMResponse::Text(content))
    }
}

// Helper function
fn convert_to_anthropic_message(msg: &ChatMessage) -> Value {
    match msg.role.as_str() {
        "system" => json!({
            "role": "user",
            "content": format!("System: {}", msg.content)
        }),
        role => json!({
            "role": role,
            "content": msg.content
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_conversion() {
        let msg = ChatMessage::user("Hello");
        let anthropic_msg = convert_to_anthropic_message(&msg);
        
        assert_eq!(anthropic_msg["role"], "user");
        assert_eq!(anthropic_msg["content"], "Hello");
    }
}