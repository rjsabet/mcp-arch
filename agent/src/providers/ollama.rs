use ollama_rs::{
    Ollama, 
    generation::chat::{ChatMessage as OllamaChatMessage},
    generation::chat::request::ChatMessageRequest
};

use crate::providers::LLMProvider;
use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;
use crate::types::{ChatMessage, ToolDefinition, LLMResponse};

pub struct OllamaProvider {
    client: Ollama,
    model: String,
}

impl OllamaProvider { // Constructor
    pub fn new(host: &str, model: &str) -> Self {
        Self {
            client: Ollama::new(host.to_string(), 11434),
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate_response(&self, messages: &[ChatMessage], _tools: &[ToolDefinition]) -> Result<LLMResponse, Box<dyn Error>> {
        // Convert messages to Ollama format
        let ollama_messages: Vec<OllamaChatMessage> = messages.iter()
            .map(|msg| convert_to_ollama_message(msg))
            .collect();

        // Create and send request
        let request = ChatMessageRequest::new(self.model.clone(), ollama_messages);
        let response = self.client.send_chat_messages(request).await?;
        
        // Return the response as text (tool calling can be handled at higher level)
        Ok(LLMResponse::Text(response.message.content))
    }
}

// Helper functions (the missing pieces from the original code)
fn convert_to_ollama_message(msg: &ChatMessage) -> OllamaChatMessage {
    match msg.role.as_str() {
        "user" => OllamaChatMessage::user(msg.content.clone()),
        "assistant" => OllamaChatMessage::assistant(msg.content.clone()),
        "system" => OllamaChatMessage::system(msg.content.clone()),
        _ => OllamaChatMessage::user(msg.content.clone()),
    }
}

#[allow(unused)]
fn convert_tools_to_ollama_format(_tools: &[ToolDefinition]) -> Vec<Value> {
    // Placeholder - tools can be handled at the agent level if needed
    vec![]
}

#[allow(unused)]
fn parse_ollama_response(response: String) -> Result<LLMResponse, Box<dyn Error>> {
    // Simple implementation - just return as text
    Ok(LLMResponse::Text(response))
}