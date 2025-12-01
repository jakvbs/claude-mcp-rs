# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **claude-mcp-rs**, a Rust implementation of an MCP (Model Context Protocol) server that wraps the Claude CLI. It enables Claude Code to invoke the local Claude CLI for AI-assisted coding tasks through the MCP protocol.

Related implementations in this workspace:
- `codexmcp/` - Python implementation with session persistence and parallel execution
- `codex-mcp-go/` - Go implementation
- `geminimcp/` - Python MCP server for Gemini CLI

## Build and Development Commands

### Building
```bash
cargo build              # Build in debug mode
cargo build --release    # Build optimized binary
```

### Running
```bash
cargo run                # Run the MCP server (listens on stdio)
```

### Testing
```bash
cargo test               # Run all tests
cargo test --lib         # Run library tests only
```

### Code Quality
```bash
cargo check              # Fast compilation check without producing binary
cargo clippy             # Lint with clippy
cargo fmt                # Format code
```

## Architecture

### Entry Point and Server Setup
The application follows a simple architecture:

1. **main.rs** - Entry point that initializes the MCP server with stdio transport
2. **server.rs** - Defines the `claude` MCP tool and handles parameter validation
3. **claude.rs** - Core Claude CLI wrapper that spawns processes and parses output
4. **lib.rs** - Module declarations

### Data Flow

```
Claude Code (MCP Client)
    ↓
stdio transport
    ↓
MCP Server (main.rs) → server::claude() tool
    ↓
claude::run() → spawns `claude` subprocess
    ↓
Parses JSON-streamed output line-by-line
    ↓
Returns ClaudeResult with session_id, agent_messages, all_messages
```

### Key Components

**server.rs:claude()** - MCP tool function that:
- Validates required parameters (PROMPT, cd)
- Validates working directory exists and is a directory
-- Constructs a minimal `Options` (prompt, working_dir, SESSION_ID, additional_args)
-- Calls `claude::run()` and formats response as `ClaudeOutput`

**claude.rs:run()** - Core execution function that:
-- Builds the `claude` command with proper arguments (`--print --output-format stream-json`, optional `--resume`)
- Spawns subprocess with stdin=null, stdout/stderr=piped
- Streams stdout line-by-line, parsing JSON events
-- Extracts `session_id` (returned as SESSION_ID), assistant `text` content, and error types
-- Returns `ClaudeResult` with all collected data

### Important Implementation Details

**Session Management**: The `SESSION_ID` (Claude's `session_id`) enables multi-turn conversations. The server extracts it from JSON output and returns it to the client for subsequent calls.

**Error Handling**: The code checks for:
- Empty SESSION_ID (indicates failed session initialization)
- Empty agent_messages (indicates no response from Claude)
- Non-zero exit codes from the Claude subprocess
- JSON parse errors in streamed output

**Streaming Output**: The Claude CLI outputs JSONL (JSON Lines) when run with `--output-format stream-json`. The server reads line-by-line to handle potentially long-running operations and collect all assistant messages incrementally.

## Dependencies

The project uses:
- **rmcp** - Official Rust MCP SDK from `modelcontextprotocol/rust-sdk`
- **tokio** - Async runtime (required by rmcp)
- **serde/serde_json** - Serialization for MCP protocol and Claude output parsing
- **anyhow** - Error handling
- **uuid** - Session ID handling

## Claude CLI Integration

This server wraps the `claude` command. Key flags used:
- `--print` - Prints a single response and exits (non-interactive mode)
- `--output-format stream-json` - Enables JSON output streaming
- `--resume <session_id>` - Continues previous session when SESSION_ID is provided
- `<prompt>` - The task prompt (positional, passed last)

Additional Claude CLI flags such as `--model`, `--permission-mode`,
`--system-prompt`, or `--strict-mcp-config` can be configured globally via
`default_additional_args()` in `src/claude.rs` so they apply to every invocation.

## Testing Strategy

The repository includes unit and integration tests for the Claude wrapper and
server. When extending tests, consider:
- Additional integration tests that mock the Claude CLI subprocess
- Validation tests for parameter handling (server.rs)
- JSON parsing tests for various Claude output formats
