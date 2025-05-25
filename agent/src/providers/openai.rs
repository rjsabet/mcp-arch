use reqwest::Client;
use serde_json::{json, Value};
use crate::providers::LLMProvider;
use crate::types::{ChatMessage, ToolDefinition, LLMResponse};
use std::error::Error;
use async_trait::async_trait;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIProvider { // Constructor
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}


#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn generate_response(&self, messages: &[ChatMessage], __tools: &[ToolDefinition]) -> Result<LLMResponse, Box<dyn Error>> {
        let request_body = json!({
            "model": self.model,
            "messages": convert_to_openai_messages(messages)
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?
            .json::<Value>()
            .await?;

        // Extract the content from the OpenAI response
            let content = response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
        
        Ok(LLMResponse::Text(content))

    }
}

// Helper function to convert ChatMessage to OpenAI format
fn convert_to_openai_messages(messages: &[ChatMessage]) -> Vec<Value> {
    messages.iter()
        .map(|msg| convert_to_openai_message(msg))
        .collect()
}

fn convert_to_openai_message(msg: &ChatMessage) -> Value {
    json!({
        "role": msg.role,
        "content": msg.content
    })
}
