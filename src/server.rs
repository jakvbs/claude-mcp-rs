use crate::claude::{self, Options};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Input parameters for claude tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClaudeArgs {
    /// Instruction for task to send to Claude
    #[serde(rename = "PROMPT")]
    pub prompt: String,
    /// Resume a previously started Claude CLI session. Must be the exact
    /// `SESSION_ID` string returned by an earlier `claude` tool call (typically
    /// a UUID). If omitted, a new session is created. Do not pass custom labels
    /// here.
    #[serde(rename = "SESSION_ID", default)]
    pub session_id: Option<String>,
}

/// Output from the claude tool
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ClaudeOutput {
    success: bool,
    #[serde(rename = "SESSION_ID")]
    session_id: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_messages_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_messages: Option<Vec<HashMap<String, Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    all_messages_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<String>,
}

#[derive(Clone)]
pub struct ClaudeServer {
    tool_router: ToolRouter<ClaudeServer>,
}

impl Default for ClaudeServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl ClaudeServer {
    /// Executes a non-interactive Claude session via CLI to perform AI-assisted coding tasks.
    /// This tool wraps the `claude` command, enabling model-driven code generation, debugging,
    /// or automation based on natural language prompts, and supports resuming ongoing sessions for continuity.
    #[tool(
        name = "claude",
        description = "Execute Claude CLI for AI-assisted coding tasks"
    )]
    async fn claude(
        &self,
        Parameters(args): Parameters<ClaudeArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate required parameters
        if args.prompt.is_empty() {
            return Err(McpError::invalid_params(
                "PROMPT is required and must be a non-empty string",
                None,
            ));
        }

        // Resolve and validate working directory based on the current process directory.
        let working_dir = std::env::current_dir().map_err(|e| {
            McpError::invalid_params(
                format!("failed to resolve current working directory: {}", e),
                None,
            )
        })?;
        let canonical_working_dir = working_dir.canonicalize().map_err(|e| {
            McpError::invalid_params(
                format!(
                    "working directory does not exist or is not accessible: {} ({})",
                    working_dir.display(),
                    e
                ),
                None,
            )
        })?;

        if !canonical_working_dir.is_dir() {
            return Err(McpError::invalid_params(
                format!(
                    "working directory is not a directory: {}",
                    working_dir.display()
                ),
                None,
            ));
        }

        // Create options for Claude CLI client
        let opts = Options {
            prompt: args.prompt,
            working_dir: canonical_working_dir,
            session_id: args.session_id,
            additional_args: claude::default_additional_args(),
            timeout_secs: None,
        };

        // Execute claude
        let result = claude::run(opts).await.map_err(|e| {
            McpError::internal_error(format!("Failed to execute claude: {}", e), None)
        })?;

        let combined_warnings = result.warnings.clone();

        // Prepare the response
        if result.success {
            let output = ClaudeOutput {
                success: true,
                session_id: result.session_id,
                message: result.agent_messages.clone(),
                agent_messages_truncated: if result.agent_messages_truncated {
                    Some(true)
                } else {
                    None
                },
                all_messages: None,
                all_messages_truncated: None,
                error: result.error.clone(),
                warnings: combined_warnings.clone(),
            };

            let json_output = serde_json::to_string(&output).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize output: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![Content::text(json_output)]))
        } else {
            // On failure, return structured error with warnings separated
            let output = ClaudeOutput {
                success: false,
                session_id: result.session_id,
                message: result.agent_messages.clone(),
                agent_messages_truncated: if result.agent_messages_truncated {
                    Some(true)
                } else {
                    None
                },
                all_messages: None,
                all_messages_truncated: None,
                error: result.error.clone(),
                warnings: combined_warnings.clone(),
            };

            let json_output = serde_json::to_string(&output).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize output: {}", e), None)
            })?;

            // Return the structured error as content instead of throwing an error
            // This allows clients to access both error and warnings fields
            Ok(CallToolResult::success(vec![Content::text(json_output)]))
        }
    }
}

#[tool_handler]
impl ServerHandler for ClaudeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a claude tool for AI-assisted coding tasks. Use the claude tool to execute coding tasks via the Claude CLI.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
}
