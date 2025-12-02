# claude-mcp-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

NPM package for **claude-mcp-rs** - A high-performance Rust implementation of an MCP (Model Context Protocol) server that wraps the Claude CLI.

## Installation

```bash
npm install -g claude-mcp-rs
```

This will automatically download and install the appropriate binary for your platform (Linux, macOS, or Windows).

## Usage with Claude Code

After installation, add to your Claude Code MCP configuration:

```bash
claude mcp add claude-rs -s user --transport stdio -- claude-mcp-rs
```

Or manually add to your `~/.claude/settings.json`:

```json
{
  "mcpServers": {
    "claude-rs": {
      "command": "claude-mcp-rs",
      "transport": "stdio"
    }
  }
}
```

## Features

- High-performance Rust implementation
- Low memory footprint
- Configurable Claude CLI flags via server config
- Session management for multi-turn conversations
- Fast async I/O with Tokio

## Supported Platforms

- Linux (x86_64, arm64)
- macOS (x86_64, arm64)
- Windows (x86_64, arm64)

## Prerequisites

You must have the [Claude CLI](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) installed and configured on your system.

## Tool Parameters

The server provides a `claude` tool with a minimal parameter surface:

- **PROMPT** (required): Task instruction
- **SESSION_ID** (optional): Resume a previously started Claude CLI session.
  Use exactly the `SESSION_ID` value returned from an earlier `claude` tool call.
  When starting a new session, omit this field entirely instead of passing an
  empty string.

Other Claude CLI flags such as `--model`, `--permission-mode`, `--system-prompt`,
and `--strict-mcp-config` are not MCP tool parameters. Configure them globally
in `claude-mcp.config.json` via `additional_args`.

## Documentation

For detailed documentation, see the [GitHub repository](https://github.com/jakvbs/claude-mcp-rs).

## License

MIT License

## Related Projects

- [codex-mcp-rs](https://github.com/jakvbs/codex-mcp-rs) - Rust MCP server for Codex CLI
- [gemini-mcp-rs](https://github.com/jakvbs/gemini-mcp-rs) - Rust MCP server for Gemini CLI
