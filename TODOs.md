| File | Line | TODO Comment |
|------|------|-------------|

## In file TODOs
| `agent/src/agent.rs` | 34 | `// TODO: Implement tool discovery from MCP server` |

## Missing/Incomplete Implementations

### 1. **`agent/providers/anthropic.rs`** - Completely Empty [x]
- **Status**: 0 bytes, no implementation
- **Needs**: Complete Anthropic Claude provider implementation

### 2. **`agent/providers/ollama.rs`** - Missing Helper Functions
- **Missing functions**:
  - `convert_to_ollama_message()` - Convert ChatMessage to OllamaChatMessage
  - `convert_tools_to_ollama_format()` - Convert ToolDefinition to Ollama tools
  - `parse_ollama_response()` - Parse Ollama response to LLMResponse
- **Missing**: Constructor function `new()` is not public
- **Missing**: `LLMProvider` trait import

### 3. **`agent/providers/openai.rs`** - Missing Helper Functions and Constructor
- **Missing functions**:
  - `convert_to_openai_messages()` - Convert ChatMessage to OpenAI format
  - `convert_tools_to_openai_format()` - Convert ToolDefinition to OpenAI tools
  - `parse_openai_response()` - Parse OpenAI response to LLMResponse
- **Missing**: Constructor function `new()`
- **Missing**: Imports for types and traits

### 4. **`agent/providers/mod.rs`** - Missing Exports
- **Missing**: Public re-exports of provider structs
- **Missing**: LLMProvider trait definition or import

### 5. **`agent/src/agent.rs`** - Tool Discovery
- **Line 34**: Tool discovery is hardcoded with only the "hello" tool
- **Needs**: Dynamic tool discovery from MCP server via JSON-RPC

## Priority Implementation Order

1. **High Priority**:
   - Fix provider implementations (missing helper functions)
   - Implement proper imports and exports in mod.rs
   - Add constructor functions for providers

2. **Medium Priority**:
   - Implement dynamic tool discovery from MCP server
   - Add Anthropic provider implementation

3. **Low Priority**:
   - Add error handling improvements
   - Add logging and debugging features

The main blocking issue is that the provider implementations are incomplete - they reference functions that don't exist, which will prevent the code from compiling.
