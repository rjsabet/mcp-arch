//! MCP Client for mcp-arch project with SSE transport

use clap::Parser;
use mcpr::{
    error::MCPError,
    schema::json_rpc::{JSONRPCMessage, JSONRPCRequest, RequestId},
    transport::{
        sse::SSETransport,
        Transport,
    },
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::error::Error;
use std::io::{self, Write};
use log::{info, error, debug};

/// CLI arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug output
    #[arg(short, long)]
    debug: bool,
    
    /// Server URI
    #[arg(short, long, default_value = "http://localhost:8080")]
    uri: String,
    
    /// Run in interactive mode
    #[arg(short, long)]
    interactive: bool,
    
    /// Name to greet (for non-interactive mode)
    #[arg(short, long)]
    name: Option<String>,
}

/// High-level MCP client
struct Client<T: Transport> {
    transport: T,
    next_request_id: i64,
}

impl<T: Transport> Client<T> {
    /// Create a new MCP client with the given transport
    fn new(transport: T) -> Self {
        Self {
            transport,
            next_request_id: 1,
        }
    }

    /// Initialize the client
    fn initialize(&mut self) -> Result<Value, MCPError> {
        // Start the transport
        debug!("Starting transport");
        self.transport.start()?;

        // Send initialization request
        let initialize_request = JSONRPCRequest::new(
            self.next_request_id(),
            "initialize".to_string(),
            Some(serde_json::json!({
                "protocol_version": mcpr::constants::LATEST_PROTOCOL_VERSION
            })),
        );

        let message = JSONRPCMessage::Request(initialize_request);
        debug!("Sending initialize request: {:?}", message);
        self.transport.send(&message)?;

        // Wait for response
        info!("Waiting for initialization response");
        let response: JSONRPCMessage = self.transport.receive()?;
        debug!("Received response: {:?}", response);

        match response {
            JSONRPCMessage::Response(resp) => Ok(resp.result),
            JSONRPCMessage::Error(err) => {
                error!("Initialization failed: {:?}", err);
                Err(MCPError::Protocol(format!(
                    "Initialization failed: {:?}",
                    err
                )))
            }
            _ => {
                error!("Unexpected response type");
                Err(MCPError::Protocol("Unexpected response type".to_string()))
            }
        }
    }

    /// Call a tool on the server
    fn call_tool<P: Serialize + std::fmt::Debug, R: DeserializeOwned>(
        &mut self,
        tool_name: &str,
        params: &P,
    ) -> Result<R, MCPError> {
        // Create tool call request
        let tool_call_request = JSONRPCRequest::new(
            self.next_request_id(),
            "tool_call".to_string(),
            Some(serde_json::json!({
                "name": tool_name,
                "parameters": serde_json::to_value(params)?
            })),
        );

        let message = JSONRPCMessage::Request(tool_call_request);
        info!("Calling tool '{}' with parameters: {:?}", tool_name, params);
        debug!("Sending tool call request: {:?}", message);
        self.transport.send(&message)?;

        // Wait for response
        info!("Waiting for tool call response");
        let response: JSONRPCMessage = self.transport.receive()?;
        debug!("Received response: {:?}", response);

        match response {
            JSONRPCMessage::Response(resp) => {
                // Extract the tool result from the response
                let result_value = resp.result;
                
                // Parse the result
                debug!("Parsing result: {:?}", result_value);
                serde_json::from_value(result_value).map_err(|e| {
                    error!("Failed to parse result: {}", e);
                    MCPError::Serialization(e)
                })
            }
            JSONRPCMessage::Error(err) => {
                error!("Tool call failed: {:?}", err);
                Err(MCPError::Protocol(format!("Tool call failed: {:?}", err)))
            }
            _ => {
                error!("Unexpected response type");
                Err(MCPError::Protocol("Unexpected response type".to_string()))
            }
        }
    }

    /// Shutdown the client
    fn shutdown(&mut self) -> Result<(), MCPError> {
        // Send shutdown request
        let shutdown_request =
            JSONRPCRequest::new(self.next_request_id(), "shutdown".to_string(), None);

        let message = JSONRPCMessage::Request(shutdown_request);
        info!("Sending shutdown request");
        debug!("Shutdown request: {:?}", message);
        self.transport.send(&message)?;

        // Wait for response
        info!("Waiting for shutdown response");
        let response: JSONRPCMessage = self.transport.receive()?;
        debug!("Received response: {:?}", response);

        match response {
            JSONRPCMessage::Response(_) => {
                // Close the transport
                info!("Closing transport");
                self.transport.close()?;
                Ok(())
            }
            JSONRPCMessage::Error(err) => {
                error!("Shutdown failed: {:?}", err);
                Err(MCPError::Protocol(format!("Shutdown failed: {:?}", err)))
            }
            _ => {
                error!("Unexpected response type");
                Err(MCPError::Protocol("Unexpected response type".to_string()))
            }
        }
    }

    /// Generate the next request ID
    fn next_request_id(&mut self) -> RequestId {
        let id = self.next_request_id;
        self.next_request_id += 1;
        RequestId::Number(id)
    }
}

fn prompt_input(prompt: &str) -> Result<String, io::Error> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Set log level based on debug flag
    if args.debug {
        log::set_max_level(log::LevelFilter::Debug);
        debug!("Debug logging enabled");
    }
    
    // Create transport and client
    info!("Using SSE transport with URI: {}", args.uri);
    let transport = SSETransport::new(&args.uri);
    
    let mut client = Client::new(transport);
    
    // Initialize the client
    info!("Initializing client...");
    let _init_result = match client.initialize() {
        Ok(result) => {
            info!("Server info: {:?}", result);
            result
        },
        Err(e) => {
            error!("Failed to initialize client: {}", e);
            return Err(Box::new(e));
        }
    };
    
    if args.interactive {
        // Interactive mode
        info!("=== mcp-arch-client Interactive Mode ===");
        println!("=== mcp-arch-client Interactive Mode ===");
        println!("Type 'exit' or 'quit' to exit");
        
        loop {
            let name = prompt_input("Enter your name (or 'exit' to quit)")?;
            if name.to_lowercase() == "exit" || name.to_lowercase() == "quit" {
                info!("User requested exit");
                break;
            }
            
            // Call the hello tool
            let request = serde_json::json!({
                "name": name
            });
            
            match client.call_tool::<Value, Value>("hello", &request) {
                Ok(response) => {
                    if let Some(message) = response.get("message") {
                        let msg = message.as_str().unwrap_or("");
                        info!("Received message: {}", msg);
                        println!("{}", msg);
                    } else {
                        info!("Received response without message field: {:?}", response);
                        println!("Response: {:?}", response);
                    }
                },
                Err(e) => {
                    error!("Error calling tool: {}", e);
                    eprintln!("Error: {}", e);
                }
            }
            
            println!();
        }
        
        info!("Exiting interactive mode");
        println!("Exiting interactive mode");
    } else {
        // One-shot mode
        let name = args.name.ok_or_else(|| {
            error!("Name is required in non-interactive mode");
            "Name is required in non-interactive mode"
        })?;
        
        info!("Running in one-shot mode with name: {}", name);
        
        // Call the hello tool
        let request = serde_json::json!({
            "name": name
        });
        
        let response: Value = match client.call_tool("hello", &request) {
            Ok(response) => response,
            Err(e) => {
                error!("Error calling tool: {}", e);
                return Err(Box::new(e));
            }
        };
        
        if let Some(message) = response.get("message") {
            let msg = message.as_str().unwrap_or("");
            info!("Received message: {}", msg);
            println!("{}", msg);
        } else {
            info!("Received response without message field: {:?}", response);
            println!("Response: {:?}", response);
        }
    }
    
    // Shutdown the client
    info!("Shutting down client");
    if let Err(e) = client.shutdown() {
        error!("Error during shutdown: {}", e);
    }
    info!("Client shutdown complete");
    
    Ok(())
}