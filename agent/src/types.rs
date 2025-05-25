use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub parameters: Value,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

pub enum LLMResponse {
    Text(String),
    ToolCall(ToolCall),
}

impl ChatMessage {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool_call(tool_call: &ToolCall) -> Self {
        Self {
            role: "assistant".to_string(),
            content: "".to_string(),
            tool_calls: Some(vec![tool_call.clone()]),
            tool_call_id: None,
        }
    }

    pub fn tool_result(result: &Value) -> Self {
        Self {
            role: "tool".to_string(),
            content: result.to_string(),
            tool_calls: None,
            tool_call_id: Some("tool_result".to_string()),
        }
    }
}