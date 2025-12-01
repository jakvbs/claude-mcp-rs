# claude-mcp-rs

[![CI](https://github.com/missdeer/claude-mcp-rs/workflows/CI/badge.svg)](https://github.com/missdeer/claude-mcp-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

A high-performance Rust implementation of an MCP (Model Context Protocol) server that wraps the Claude CLI for AI-assisted coding tasks.

## Features

- **MCP Protocol Support**: Implements the official Model Context Protocol using the Rust SDK
- **Claude Integration**: Wraps the Claude CLI to enable AI-assisted coding through MCP
- **Session Management**: Supports multi-turn conversations via session IDs
- **Sandbox Safety**: Configurable sandbox policies (read-only, workspace-write, danger-full-access)
- **Async Runtime**: Built on Tokio for efficient async I/O

## Prerequisites

- Rust 1.70+ (uses 2021 edition)
- [Claude CLI](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code-cli) installed and configured
- Claude Code or another MCP client

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

## Installation

### Option 1: Install via NPM (Recommended)

The easiest way to install is via npm, which will automatically download the correct binary for your platform:

```bash
npm install -g @missdeer/claude-mcp-rs
```

Then add to your Claude Code MCP configuration:

```bash
claude mcp add claude-rs -s user --transport stdio -- claude-mcp-rs
```

### Option 2: Install from Release

Download the appropriate binary for your platform from the [releases page](https://github.com/missdeer/claude-mcp-rs/releases), extract it, and add to your MCP configuration:

```bash
claude mcp add claude-rs -s user --transport stdio -- /path/to/claude-mcp-rs
```

### Option 3: Build from Source

```bash
git clone https://github.com/missdeer/claude-mcp-rs.git
cd claude-mcp-rs
cargo build --release
claude mcp add claude-rs -s user --transport stdio -- $(pwd)/target/release/claude-mcp-rs
```

## Tool Usage

The server provides a single `claude` tool with a deliberately small parameter
surface. Most Claude CLI flags are configured globally in the server rather
than exposed as MCP parameters.

### Required Parameters

- `PROMPT` (string): Task instruction for Claude

### Optional Parameters

- `SESSION_ID` (string): Resume a previously started Claude CLI session for
  multi-turn conversations. Use exactly the `SESSION_ID` value returned from an
  earlier `claude` tool call (typically a UUID). If omitted, a new session is
  created. Do not pass custom labels here.

## Configuration (JSON)

The server can load additional Claude CLI arguments and a default timeout from
`claude-mcp.config.json` in the current working directory, or from a path
specified via the `CLAUDE_MCP_CONFIG_PATH` environment variable.

Example:

```json
{
  "additional_args": [
    "--dangerously-bypass-approvals-and-sandbox",
    "--profile",
    "gpt-5"
  ],
  "timeout_secs": 600
}
```

`additional_args` are appended to every Claude CLI invocation after the core
flags (`--print`, `--output-format stream-json`) and before any `--resume` / prompt arguments.
`timeout_secs` controls the maximum runtime for each Claude execution:
- omitted or <= 0 → defaults to 600 seconds,
- values above 3600 are clamped to 3600 seconds.

## Testing

The project has comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# See detailed testing guide
cat TESTING.md
```

Test categories:
- **Unit tests** (10): Core functionality (escape_prompt, Options)
- **Integration tests** (10): End-to-end scenarios
- **Server tests** (5): MCP protocol implementation
- **CI tests**: Multi-platform validation

Total: 25 tests passing ✅

Current test coverage: See [Codecov](https://codecov.io/gh/missdeer/claude-mcp-rs)

## Architecture

See [CLAUDE.md](./CLAUDE.md) for detailed architecture documentation.

## Comparison with Other Implementations

| Feature | claude-mcp-rs (Rust) | codexmcp (Python) | codex-mcp-go |
|---------|---------------------|-------------------|--------------|
| Language | Rust | Python | Go |
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| Memory Usage | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| Binary Size | Medium | N/A | Small |
| Startup Time | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Session Management | ✓ | ✓ | ✓ |
| Image Support | ✗ | ✓ | ✓ |
| Sandbox Policies | ✓ | ✓ | ✓ |

## Related Projects

- [codexmcp](https://github.com/GuDaStudio/codexmcp) - Original Python implementation by guda.studio
- [codex-mcp-go](https://github.com/w31r4/codex-mcp-go) - Go implementation
- [geminimcp](https://github.com/GuDaStudio/geminimcp) - Python MCP server for Gemini CLI

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - Copyright (c) 2025 missdeer

See [LICENSE](./LICENSE) for details.
