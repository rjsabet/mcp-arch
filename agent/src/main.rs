use clap::Parser;
use crate::providers::*;
use crate::agent::LLMAgent;
//use mcp_arch_agent::{LLMAgent, providers::*};
use mcpr::transport::sse::SSETransport;
use mcpr::client::Client;
use std::io::{self, Write};
use std::error::Error;

mod providers;
mod types;
mod agent;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// MCP Server URI
    #[arg(long, default_value = "http://localhost:8080")]
    mcp_server_uri: String,
    
    /// LLM Provider (ollama, openai, claude)
    #[arg(long, default_value = "ollama")]
    llm_provider: String,
    
    /// Model name
    #[arg(long, default_value = "llama2")]
    model: String,
    
    /// Ollama host (for ollama provider)
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_host: String,
    
    /// OpenAI API key (for openai provider)
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let args = Args::parse();
    
    // Create MCP client
    let transport = SSETransport::new(&args.mcp_server_uri);
    let mcp_client = Client::new(transport);
    
    // Create LLM provider based on configuration
    let llm_provider: Box<dyn LLMProvider> = match args.llm_provider.as_str() {
        "ollama" => Box::new(OllamaProvider::new(&args.ollama_host, &args.model)),
        "openai" => {
            let api_key = args.openai_api_key
                .ok_or("OpenAI API key required for OpenAI provider")?;
            Box::new(OpenAIProvider::new(&api_key, &args.model))
        },
        _ => return Err("Unsupported LLM provider".into()),
    };
    
    // Create and initialize agent
    let mut agent = LLMAgent::new(llm_provider, mcp_client);
    agent.initialize().await?;
    
    println!("🤖 LLM Agent ready! Type 'exit' to quit.");
    
    // Interactive chat loop
    loop {
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("exit") {
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        match agent.chat(input).await {
            Ok(response) => println!("Agent: {}", response),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    
    println!("Goodbye!");
    Ok(())
}