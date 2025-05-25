pub mod ollama;
pub mod openai;
pub mod anthropic;

pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;

use crate::types::*;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn generate_response(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<LLMResponse, Box<dyn Error>>;
}