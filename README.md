# mcp-arch - MCP Project

A complete MCP project with both client and server components, using SSE transport.

## Features

- **Server-Sent Events Transport**: HTTP-based transport with proper client registration and message handling
- **Multiple Client Support**: Server can handle multiple clients simultaneously
- **Interactive Mode**: Choose tools and provide parameters interactively
- **One-shot Mode**: Run queries directly from the command line
- **Comprehensive Logging**: Detailed logging for debugging and monitoring

## Project Structure

- `client/`: The MCP client implementation
- `server/`: The MCP server implementation with tools
- `test.sh`: A test script to run both client and server

## Building

```bash
# Build the server
cd server
cargo build

# Build the client
cd ../client
cargo build
```

## Running

### Start the Server

```bash
cd server
cargo run -- --port 8080
```

### Run the Client

```bash
cd client
cargo run -- --uri "http://localhost:8080" --name "Your Name"
```

### Interactive Mode

```bash
cd client
cargo run -- --uri "http://localhost:8080" --interactive
```

## Testing

```bash
./test.sh
```

## Available Tools

- `hello`: A simple tool that greets a person by name
