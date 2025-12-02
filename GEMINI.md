# Gemini Code Assistant Context

This document provides context for the Gemini code assistant to understand the `claude-mcp-rs` project.

## Project Overview

`claude-mcp-rs` is a high-performance, open-source server for the **Model Context Protocol (MCP)**, written in **Rust**. It acts as a wrapper around the Claude CLI (a command-line tool for AI-assisted coding), enabling it to communicate with MCP-compatible clients like the Claude Code IDE extension.

The server is built using the official Rust MCP SDK (`rmcp`) and leverages the `tokio` runtime for asynchronous I/O, ensuring efficient and non-blocking communication. It provides a single `claude` tool that clients can use to execute tasks.

### Core Technologies

*   **Language:** Rust (2021 Edition)
*   **Main Dependencies:**
    *   `rmcp`: The official Rust SDK for the Model Context Protocol.
    *   `tokio`: Asynchronous runtime for high-performance I/O.
    *   `serde`: Framework for serializing and deserializing Rust data structures.
    *   `anyhow`: Flexible error handling library.

### Architecture

*   **Entry Point:** The application starts in `src/main.rs`, which initializes and runs the `ClaudeServer`.
*   **Server Logic:** `src/server.rs` contains the core implementation of the `ClaudeServer`, which handles MCP requests and dispatches them to the Claude CLI.
*   **Claude CLI Wrapper:** `src/claude.rs` defines the `Claude` struct and its methods, which are responsible for constructing and executing commands for the Claude CLI.
*   **Library:** `src/lib.rs` is the library crate root, making the server implementation available to the `main` binary.

## Building and Running

### Building the Project

The project is built using `cargo`, the Rust build tool.

*   **Debug Build:**
    ```bash
    cargo build
    ```
*   **Release Build:**
    ```bash
    cargo build --release
    ```

### Running the Server

The server communicates over `stdio`.

*   **Run with Cargo:**
    ```bash
    cargo run
    ```
*   **Run compiled binary:**
    ```bash
    ./target/release/claude-mcp-rs
    ```

## Development Conventions

### Testing

The project has a comprehensive test suite.

*   **Run all tests:**
    ```bash
    cargo test
    ```

Tests are organized into three categories:

*   **Unit Tests:** Located alongside the code they are testing.
*   **Integration Tests:** `tests/integration_tests.rs`
*   **Server Tests:** `tests/server_tests.rs`

### Linting and Formatting

*   **Format code:**
    ```bash
    cargo fmt
    ```
*   **Lint code:**
    ```bash
    cargo clippy
    ```
