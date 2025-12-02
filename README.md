# claude-mcp-rs

[![CI](https://github.com/jakvbs/claude-mcp-rs/workflows/CI/badge.svg)](https://github.com/jakvbs/claude-mcp-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

A high-performance Rust implementation of an MCP (Model Context Protocol) server that wraps the Claude CLI for AI-assisted coding tasks.

## Features

- **MCP Protocol Support**: Implements the official Model Context Protocol using the Rust SDK
- **Claude CLI Integration**: Wraps the Claude CLI to enable AI-assisted coding through MCP
- **Session Management**: Supports multi-turn conversations via session IDs
- **Configurable**: CLI arguments and timeout configurable via JSON config file
- **Async Runtime**: Built on Tokio for efficient async I/O

## Prerequisites

- Rust 1.70+ (uses 2021 edition)
- [Claude CLI](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) installed and configured
- Claude Code or another MCP client

## Installation

### Option 1: Build from Source

```bash
git clone https://github.com/jakvbs/claude-mcp-rs.git
cd claude-mcp-rs
cargo build --release
```

Then add to your Claude Code MCP configuration:

```bash
claude mcp add claude-rs -s user --transport stdio -- /path/to/claude-mcp-rs
```

### Option 2: Download from Releases

Download the appropriate binary for your platform from the [releases page](https://github.com/jakvbs/claude-mcp-rs/releases).

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Running

The server communicates via stdio transport:

```bash
cargo run
```

Or after building:

```bash
./target/release/claude-mcp-rs
```

## Tool Usage

The server provides a single `claude` tool with a minimal parameter surface. Most Claude CLI flags are configured globally via the config file.

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `PROMPT` | string | Task instruction for Claude |

### Optional Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `SESSION_ID` | string | Resume a previously started Claude CLI session. Use the exact `SESSION_ID` value returned from an earlier call. If omitted, a new session is created. Do not send an empty string value: when starting a new session, omit the `SESSION_ID` field entirely instead of passing `\"\"`. |

### Response Structure

```json
{
  "success": true,
  "SESSION_ID": "uuid-string",
  "message": "Claude's response text",
  "error": null,
  "warnings": null
}
```

## Configuration

The server loads configuration from `claude-mcp.config.json` in the current working directory, or from a path specified via the `CLAUDE_MCP_CONFIG_PATH` environment variable.

Example configuration:

```json
{
  "additional_args": [
    "--dangerously-skip-permissions"
  ],
  "timeout_secs": 600
}
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `additional_args` | string[] | `[]` | Extra CLI arguments passed to every Claude invocation |
| `timeout_secs` | number | `600` | Maximum runtime per execution (clamped to 3600 max) |

The `additional_args` are appended after core flags (`--print`, `--output-format stream-json`) and before any `--resume` or prompt arguments.

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests
```

Test coverage:
- **Unit tests**: Core functionality
- **Integration tests**: End-to-end scenarios
- **Server tests**: MCP protocol implementation

## Architecture

```
Claude Code (MCP Client)
    |
stdio transport
    |
MCP Server (main.rs) -> server::claude() tool
    |
claude::run() -> spawns `claude` subprocess
    |
Parses JSON-streamed output line-by-line
    |
Returns ClaudeResult with session_id, agent_messages
```

See [CLAUDE.md](./CLAUDE.md) for detailed architecture documentation.

## Related Projects

- [codex-mcp-rs](https://github.com/jakvbs/codex-mcp-rs) - Rust MCP server for Codex CLI
- [gemini-mcp-rs](https://github.com/jakvbs/gemini-mcp-rs) - Rust MCP server for Gemini CLI

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - See [LICENSE](./LICENSE) for details.
