# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test suite (25 tests total)
  - Unit tests for Options validation
  - Integration tests for end-to-end scenarios
  - Server tests for MCP protocol implementation
- Test utilities in tests/common/mod.rs
- TESTING.md documentation with testing guide
- Enhanced CI workflow with:
  - Code coverage reporting via cargo-tarpaulin
  - Security audits via cargo-audit
  - Multi-platform testing (Ubuntu, macOS, Windows)
  - Multiple Rust versions (stable, beta)

### Changed
- Simplified MCP tool surface. The `claude` tool now accepts
  `PROMPT` (required) and optional `SESSION_ID`. Other CLI flags
  must be configured via `additional_args` in the config file.

## [0.1.0] - 2025-01-28

### Added
- Initial release of claude-mcp-rs
- MCP server implementation using official Rust SDK (rmcp)
- Claude CLI wrapper with JSON output parsing
- Session management for multi-turn conversations
- Async I/O with Tokio runtime
- Cross-platform support (Linux, macOS, Windows x x86_64, arm64)
- GitHub Actions CI/CD workflows
- Comprehensive documentation (README, CLAUDE.md, CONTRIBUTING.md, QUICKSTART.md)
- MIT License

### Features
- **Tool**: `claude` - Execute Claude CLI for AI-assisted coding tasks
  - Required parameters: `PROMPT`
  - Optional parameters: `SESSION_ID`
- **Transport**: stdio (standard input/output)
- **Error handling**: Comprehensive validation and error messages
- **Performance**: High-performance Rust implementation with low memory footprint

### Documentation
- Installation guides (binary, source)
- Usage examples and common use cases
- Architecture documentation for developers
- Contribution guidelines
- Quick start guide

### Infrastructure
- Automated multi-platform builds
- Continuous Integration testing
- Makefile for development convenience

[Unreleased]: https://github.com/jakvbs/claude-mcp-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/jakvbs/claude-mcp-rs/releases/tag/v0.1.0
