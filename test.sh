#!/bin/bash

# Test script for mcp-arch MCP project with SSE transport

# Exit on error
set -e

# Enable verbose output
set -x

# Function to clean up on exit
cleanup() {
  echo "Cleaning up..."
  if [ ! -z "$SERVER_PID" ]; then
    echo "Shutting down server (PID: $SERVER_PID)..."
    kill $SERVER_PID 2>/dev/null || true
  fi
  if [ ! -z "$CAT_PID" ]; then
    kill $CAT_PID 2>/dev/null || true
  fi
  if [ ! -z "$SERVER_PIPE" ] && [ -e "$SERVER_PIPE" ]; then
    rm $SERVER_PIPE 2>/dev/null || true
  fi
  exit $1
}

# Set up trap for clean exit
trap 'cleanup 1' INT TERM

echo "Building server..."
cd server
cargo build

echo "Building client..."
cd ../client
cargo build

# Create a named pipe for server output
SERVER_PIPE="/tmp/server_pipe_$$"
mkfifo $SERVER_PIPE

# Start reading from the pipe in the background
cat $SERVER_PIPE &
CAT_PID=$!

echo "Starting server in background..."
cd ..
RUST_LOG=debug,mcpr=trace ./server/target/debug/mcp-arch-server --port 8081 > $SERVER_PIPE 2>&1 &
SERVER_PID=$!

# Give the server time to start
echo "Waiting for server to start..."
sleep 3

# Check if server is still running
if ! kill -0 $SERVER_PID 2>/dev/null; then
  echo "Error: Server failed to start or crashed"
  cleanup 1
fi

echo "Running client..."
RUST_LOG=debug,mcpr=trace ./client/target/debug/mcp-arch-client --uri "http://localhost:8081" --name "MCP User"
CLIENT_EXIT=$?

if [ $CLIENT_EXIT -ne 0 ]; then
  echo "Error: Client exited with code $CLIENT_EXIT"
  cleanup $CLIENT_EXIT
fi

echo "Shutting down server..."
kill $SERVER_PID 2>/dev/null || true
kill $CAT_PID 2>/dev/null || true
rm $SERVER_PIPE 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo "Test completed successfully!"
cleanup 0
