use crate::types::*;
use crate::providers::LLMProvider;
use mcpr::{client::Client, transport::Transport};
use serde_json::Value;
use std::error::Error;
use log::{info, warn};


pub struct LLMAgent<T: Transport> {
    llm_provider: Box<dyn LLMProvider>,
    mcp_client: Client<T>,
    conversation_history: Vec<ChatMessage>,
    available_tools: Vec<ToolDefinition>,
}

impl<T: Transport> LLMAgent<T> {
    
    // Test tool call to verify MCP connection
    #[allow(unused)]
    pub async fn test_connection(&mut self) -> Result<(), Box<dyn Error>> {
        let test_params = serde_json::json!({"name": "Test User"});
        let result = self.mcp_client.call_tool::<Value, Value>("test_tool", &test_params)?;
        println!("MCP connection test successful:{:?}", result);
        Ok(())
    }
    
    pub fn new(llm_provider: Box<dyn LLMProvider>, mcp_client: Client<T>) -> Self {
        Self {
            llm_provider,
            mcp_client,
            conversation_history: Vec::new(),
            available_tools: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        // Initialize MCP client and capture the response
        let init_response = self.mcp_client.initialize()?;
        
        // Discover available tools from MCP server
        self.discover_tools(init_response).await?;
        
        Ok(())
    }

    pub async fn discover_tools(&mut self, init_response: Value) -> Result<(), Box<dyn Error>> {
        // Extract tools from the initialization response
        if let Some(tools_array) = init_response.get("tools").and_then(|t| t.as_array()) {
            self.available_tools.clear(); // Clear any existing tools
            
            for tool_value in tools_array {
                // Convert mcpr::Tool to our ToolDefinition
                if let Some(tool_name) = tool_value.get("name").and_then(|n| n.as_str()) {
                    let description = tool_value
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("No description available")
                        .to_string();
                    
                    // Extract the parameters schema from input_schema
                    let parameters = if let Some(input_schema) = tool_value.get("input_schema") {
                        // Convert the input_schema to parameters format expected by LLMs
                        let properties = input_schema.get("properties").cloned().unwrap_or(Value::Null);
                        let required = input_schema.get("required").cloned().unwrap_or(Value::Null);
                        
                        serde_json::json!({
                            "type": "object",
                            "properties": properties,
                            "required": required
                        })
                    } else {
                        serde_json::json!({
                            "type": "object",
                            "properties": {},
                            "required": []
                        })
                    };
                    
                    self.available_tools.push(ToolDefinition {
                        name: tool_name.to_string(),
                        description: description.clone(),
                        parameters,
                    });
                    
                    info!("Discovered tool: {} - {}", tool_name, &description);
                }
            }
            
            info!("Successfully discovered {} tools", self.available_tools.len());
        } else {
            warn!("No tools found in initialization response");
        }
        
        Ok(())
    }

    pub async fn chat(&mut self, user_message: &str) -> Result<String, Box<dyn Error>> {
        // Add user message to conversation history
        self.conversation_history.push(ChatMessage::user(user_message));
        
        loop {
            // Generate LLM response with available tools
            let llm_response = self.llm_provider
                .generate_response(&self.conversation_history, &self.available_tools)
                .await?;

            match llm_response {
                LLMResponse::Text(text) => {
                    // Return final response
                    self.conversation_history.push(ChatMessage::assistant(&text));
                    return Ok(text);
                }
                LLMResponse::ToolCall(tool_call) => {
                    // Execute tool via MCP client
                    let tool_result = self.execute_tool(&tool_call).await?;
                    
                    // Add tool call and result to conversation
                    self.conversation_history.push(ChatMessage::tool_call(&tool_call));
                    self.conversation_history.push(ChatMessage::tool_result(&tool_result));
                    
                    // Continue loop to get final response
                }
            }
        }
    }

    async fn execute_tool(&mut self, tool_call: &ToolCall) -> Result<Value, Box<dyn Error>> {
        // Call MCP client to execute the tool
        let result = self.mcp_client.call_tool(&tool_call.name, &tool_call.parameters)?;
        Ok(result)
    }
}