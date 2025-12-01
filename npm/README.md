# @missdeer/claude-mcp-rs

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

NPM package for **claude-mcp-rs** - A high-performance Rust implementation of an MCP (Model Context Protocol) server that wraps the Claude CLI.

## Installation

```bash
npm install -g @missdeer/claude-mcp-rs
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

- ✨ High-performance Rust implementation
- 🚀 Low memory footprint
- 🔒 Configurable Claude CLI flags (e.g. permission mode, model) via server config
- 🔄 Session management for multi-turn conversations
- ⚡ Fast async I/O with Tokio

## Supported Platforms

- Linux (x86_64, arm64)
- macOS (x86_64, arm64)
- Windows (x86_64, arm64)

## Prerequisites

You must have the [Claude CLI](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code-cli) installed and configured on your system.

## Tool Parameters

The server provides a `claude` tool with a minimal parameter surface:

- **PROMPT** (required): Task instruction
- **SESSION_ID** (optional): Resume a previously started Claude CLI session
  (`session_id`). Use exactly the `SESSION_ID` value returned from an earlier
  `claude` tool call; leaving it empty starts a new session.

Other Claude CLI flags such as `--model`, `--permission-mode`, `--system-prompt`,
and `--strict-mcp-config` are not MCP tool parameters. Configure them globally
in `src/claude.rs` via `default_additional_args()` so they apply to every Claude invocation.

## Documentation

For detailed documentation, see the [GitHub repository](https://github.com/missdeer/claude-mcp-rs).

## License

MIT License - Copyright (c) 2025 missdeer

## Related Projects

- [codexmcp](https://github.com/GuDaStudio/codexmcp) - Python implementation
- [codex-mcp-go](https://github.com/w31r4/codex-mcp-go) - Go implementation
- [geminimcp](https://github.com/GuDaStudio/geminimcp) - Gemini CLI MCP server
