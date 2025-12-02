# Quick Start Guide

Get started with claude-mcp-rs in 5 minutes!

## Prerequisites

1. Install [Claude CLI](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview):
   ```bash
   # Follow Claude CLI installation instructions
   claude --version
   ```

2. Install [Claude Code](https://docs.claude.com/docs/claude-code):
   ```bash
   claude --version
   ```

## Installation

### Building from Source

```bash
# Clone repository
git clone https://github.com/jakvbs/claude-mcp-rs.git
cd claude-mcp-rs

# Build release binary
cargo build --release

# Add to Claude Code
claude mcp add claude-rs -s user --transport stdio -- $(pwd)/target/release/claude-mcp-rs
```

### Using Pre-built Binary

1. Download from [releases](https://github.com/jakvbs/claude-mcp-rs/releases)
2. Extract the archive
3. Add to Claude Code:
   ```bash
   claude mcp add claude-rs -s user --transport stdio -- /path/to/claude-mcp-rs
   ```

## Verification

Check that the server is registered:

```bash
claude mcp list
```

You should see:
```
claude-rs: claude-mcp-rs - Connected
```

## Basic Usage

In Claude Code, you can now use the `claude` tool:

```
Use the claude tool to implement a function that calculates fibonacci numbers
```

Claude Code will call the claude tool with:
```json
{
  "PROMPT": "implement a function that calculates fibonacci numbers"
}
```

## Common Use Cases

### 1. Generate Code

```
Use claude to create a REST API server in Go with CRUD operations
```

### 2. Fix Bugs

```
Use claude to debug and fix the error in src/main.rs
```

### 3. Refactor Code

```
Use claude to refactor the authentication module to use JWT
```

### 4. Multi-turn Conversation

```
First call:
Use claude to analyze the codebase structure

Second call (using SESSION_ID from first response):
Now suggest improvements to the architecture
SESSION_ID: <previous-session-id>
```

## Configuration

Create `claude-mcp.config.json` in your working directory:

```json
{
  "additional_args": ["--dangerously-skip-permissions"],
  "timeout_secs": 600
}
```

## Troubleshooting

### "Failed to execute claude"

Check Claude CLI is installed:
```bash
claude --version
```

### Server won't start

Check logs:
```bash
claude mcp logs claude-rs
```

## Next Steps

- Read [README.md](./README.md) for detailed features
- See [CLAUDE.md](./CLAUDE.md) for architecture details
- Check [CONTRIBUTING.md](./CONTRIBUTING.md) to contribute

## Getting Help

- [Report bugs](https://github.com/jakvbs/claude-mcp-rs/issues)
