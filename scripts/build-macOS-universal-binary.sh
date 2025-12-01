#!/bin/sh
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create -output claude-mcp-rs target/aarch64-apple-darwin/release/claude-mcp-rs target/x86_64-apple-darwin/release/claude-mcp-rs
