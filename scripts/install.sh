#!/bin/bash

# check the first argument is the path to the claude-mcp-rs binary
if [ -n "$1" ]; then
    CLAUDE_MCP_RS_PATH="$1"
fi

if [ -z "$CLAUDE_MCP_RS_PATH" ]; then
    # Get the absolute path of the claude-mcp-rs binary
    # if current os is Darwin, use $(pwd)/claude-mcp-rs
    if [ "$(uname)" == "Darwin" ]; then
        CLAUDE_MCP_RS_PATH=$(pwd)/claude-mcp-rs
    fi
    if [ ! -f "$CLAUDE_MCP_RS_PATH" ]; then
        CLAUDE_MCP_RS_PATH=$(pwd)/target/release/claude-mcp-rs
        if [ ! -f "$CLAUDE_MCP_RS_PATH" ]; then
            echo "Error: claude-mcp-rs binary not found"
            exit 1
        fi
    fi
fi

# Add the claude-mcp-rs server to the Claude Code MCP registry
CLAUDE_PATH=$(which claude)
if [ -f "$CLAUDE_PATH" ]; then
    "$CLAUDE_PATH" mcp add claude-rs -s user --transport stdio -- "$CLAUDE_MCP_RS_PATH"
else
    echo "Error: claude not found"
    exit 1
fi
