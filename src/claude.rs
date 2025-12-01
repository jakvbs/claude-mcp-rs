use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct Options {
    pub prompt: String,
    pub working_dir: PathBuf,
    pub session_id: Option<String>,
    /// Extra CLI flags to pass through to the Claude CLI.
    pub additional_args: Vec<String>,
    /// Timeout in seconds for the Claude execution. If None, defaults to 600 seconds (10 minutes).
    /// Set to a specific value to override. The library enforces a timeout to prevent unbounded execution.
    pub timeout_secs: Option<u64>,
}

const DEFAULT_TIMEOUT_SECS: u64 = 600;
const MAX_TIMEOUT_SECS: u64 = 3600;

/// Configuration loaded from `claude-mcp.config.json` (or `CLAUDE_MCP_CONFIG_PATH`).
#[derive(Debug, Clone, Deserialize)]
struct ServerConfig {
    #[serde(default)]
    additional_args: Vec<String>,
    timeout_secs: Option<u64>,
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("CLAUDE_MCP_CONFIG_PATH") {
        let trimmed = env_path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    // Fallback: config file in the current working directory
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("claude-mcp.config.json"))
}

fn load_server_config() -> ServerConfig {
    let mut cfg = ServerConfig {
        additional_args: Vec::new(),
        timeout_secs: None,
    };

    let Some(config_path) = resolve_config_path() else {
        return cfg;
    };

    if !config_path.is_file() {
        return cfg;
    }

    match std::fs::read_to_string(&config_path) {
        Ok(raw) => match serde_json::from_str::<ServerConfig>(&raw) {
            Ok(parsed) => {
                let mut cleaned = parsed;
                cleaned.additional_args = cleaned
                    .additional_args
                    .into_iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                cfg = cleaned;
            }
            Err(err) => {
                eprintln!(
                    "claude-mcp-rs: failed to parse config {}: {}",
                    config_path.display(),
                    err
                );
            }
        },
        Err(err) => {
            eprintln!(
                "claude-mcp-rs: failed to read config {}: {}",
                config_path.display(),
                err
            );
        }
    }

    cfg
}

fn server_config() -> &'static ServerConfig {
    static SERVER_CONFIG: OnceLock<ServerConfig> = OnceLock::new();
    SERVER_CONFIG.get_or_init(load_server_config)
}

/// Default extra CLI flags applied to every Claude CLI invocation.
/// Update configuration via `claude-mcp.config.json` or the
/// `CLAUDE_MCP_CONFIG_PATH` environment variable.
pub fn default_additional_args() -> Vec<String> {
    server_config().additional_args.clone()
}

/// Default timeout (in seconds) for Claude runs, configurable via
/// `timeout_secs` in `claude-mcp.config.json`. Values <= 0 or missing
/// fall back to 600; values above MAX_TIMEOUT_SECS are clamped.
pub fn default_timeout_secs() -> u64 {
    static CACHED_TIMEOUT: OnceLock<u64> = OnceLock::new();
    *CACHED_TIMEOUT.get_or_init(|| {
        let cfg = server_config();
        match cfg.timeout_secs {
            Some(t) if t > 0 && t <= MAX_TIMEOUT_SECS => t,
            Some(t) if t > MAX_TIMEOUT_SECS => MAX_TIMEOUT_SECS,
            _ => DEFAULT_TIMEOUT_SECS,
        }
    })
}

#[derive(Debug)]
pub struct ClaudeResult {
    pub success: bool,
    pub session_id: String,
    pub agent_messages: String,
    pub agent_messages_truncated: bool,
    pub all_messages: Vec<HashMap<String, Value>>,
    pub all_messages_truncated: bool,
    pub error: Option<String>,
    pub warnings: Option<String>,
}

/// Result of reading a line with length limit
#[derive(Debug)]
struct ReadLineResult {
    bytes_read: usize,
    truncated: bool,
}

/// Validation mode for enforce_required_fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
    /// Perform full validation (check session_id and agent_messages)
    Full,
    /// Skip validation (for cases with well-defined errors like timeout or truncation)
    Skip,
}

/// Read a line from an async buffered reader with a maximum length limit to prevent memory spikes
/// Returns the number of bytes read (0 on EOF) and whether the line was truncated
/// Reads in chunks and enforces max_len during reading to prevent OOM from extremely long lines
///
/// After hitting max_len, continues reading until newline to properly consume the full line.
/// This ensures the next read starts at the correct position. For subprocess stdout (our use case),
/// this is appropriate because:
/// 1. The Codex CLI always outputs newline-terminated JSON
/// 2. Process-level timeout prevents indefinite blocking
/// 3. We stop allocating memory once max_len is hit, preventing OOM
async fn read_line_with_limit<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
    buf: &mut Vec<u8>,
    max_len: usize,
) -> std::io::Result<ReadLineResult> {
    let mut total_read = 0;
    let mut truncated = false;

    loop {
        // Fill the internal buffer if needed
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            break; // EOF
        }

        // Process available bytes
        for (i, &byte) in available.iter().enumerate() {
            if !truncated && buf.len() < max_len {
                buf.push(byte);
                total_read += 1;
            } else if !truncated {
                truncated = true;
            }

            if byte == b'\n' {
                reader.consume(i + 1);
                return Ok(ReadLineResult {
                    bytes_read: total_read,
                    truncated,
                });
            }
        }

        let consumed = available.len();
        reader.consume(consumed);
    }

    Ok(ReadLineResult {
        bytes_read: total_read,
        truncated,
    })
}

/// Execute Claude CLI with the given options and return the result
/// Requires timeout to be set to prevent unbounded execution
pub async fn run(mut opts: Options) -> Result<ClaudeResult> {
    // Ensure timeout is always set
    if opts.timeout_secs.is_none() {
        opts.timeout_secs = Some(default_timeout_secs());
    }

    let timeout_secs = opts.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
    let duration = std::time::Duration::from_secs(timeout_secs);

    match tokio::time::timeout(duration, run_internal(opts)).await {
        Ok(result) => result,
        Err(_) => {
            // Timeout occurred - the child process will be killed automatically via kill_on_drop
            let result = ClaudeResult {
                success: false,
                session_id: String::new(),
                agent_messages: String::new(),
                agent_messages_truncated: false,
                all_messages: Vec::new(),
                all_messages_truncated: false,
                error: Some(format!(
                    "Claude execution timed out after {} seconds",
                    timeout_secs
                )),
                warnings: None,
            };
            // Skip validation since timeout error is already well-defined
            Ok(enforce_required_fields(result, ValidationMode::Skip))
        }
    }
}

/// Internal implementation of Claude CLI execution
async fn run_internal(opts: Options) -> Result<ClaudeResult> {
    // Allow overriding the claude binary for tests or custom setups
    let claude_bin = std::env::var("CLAUDE_BIN").unwrap_or_else(|_| "claude".to_string());

    // Build the base command
    let mut cmd = Command::new(claude_bin);

    // Run in the configured working directory (Claude CLI uses the current
    // process directory as its workspace context).
    cmd.current_dir(&opts.working_dir);

    // Always request JSON-streaming output suitable for MCP
    cmd.arg("--print");
    cmd.args(["--output-format", "stream-json"]);

    // Append any extra CLI flags requested by the caller, before the prompt delimiter.
    for arg in &opts.additional_args {
        cmd.arg(arg);
    }

    // Add session resume flag when resuming an existing conversation
    if let Some(ref session_id) = opts.session_id {
        cmd.args(["--resume", session_id]);
    }

    // Add the prompt as a positional argument at the end - Command::arg()
    // handles proper escaping across platforms.
    cmd.arg(&opts.prompt);

    // Configure process
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true); // Ensure child is killed if this future is dropped (e.g., on timeout)

    // Spawn the process
    let mut child = cmd.spawn().context("Failed to spawn claude command")?;

    // Read stdout
    let stdout = child.stdout.take().context("Failed to get stdout")?;
    let stderr = child.stderr.take().context("Failed to get stderr")?;

    let mut result = ClaudeResult {
        success: true,
        session_id: String::new(),
        agent_messages: String::new(),
        agent_messages_truncated: false,
        all_messages: Vec::new(),
        all_messages_truncated: false,
        error: None,
        warnings: None,
    };

    // Spawn a task to drain stderr and capture diagnostics with better error handling
    const MAX_STDERR_SIZE: usize = 1024 * 1024; // 1MB limit for stderr
    const MAX_LINE_LENGTH: usize = 1024 * 1024; // 1MB per line to prevent memory spikes
    const MAX_AGENT_MESSAGES_SIZE: usize = 10 * 1024 * 1024; // 10MB limit for agent messages
    const MAX_ALL_MESSAGES_SIZE: usize = 50 * 1024 * 1024; // 50MB limit for all messages combined
    let stderr_handle = tokio::spawn(async move {
        let mut stderr_output = String::new();
        let mut stderr_reader = BufReader::new(stderr);
        let mut truncated = false;
        let mut line_buf = Vec::new();

        loop {
            line_buf.clear();
            match read_line_with_limit(&mut stderr_reader, &mut line_buf, MAX_LINE_LENGTH).await {
                Ok(read_result) => {
                    if read_result.bytes_read == 0 {
                        break; // EOF
                    }
                    // Convert to string, handling invalid UTF-8
                    let line = String::from_utf8_lossy(&line_buf);
                    let line = line.trim_end_matches('\n').trim_end_matches('\r');

                    // Check if adding this line would exceed the limit
                    let new_size = stderr_output.len() + line.len() + 1; // +1 for newline
                    if new_size > MAX_STDERR_SIZE {
                        if !truncated {
                            if !stderr_output.is_empty() {
                                stderr_output.push('\n');
                            }
                            stderr_output.push_str("[... stderr truncated due to size limit ...]");
                            truncated = true;
                        }
                        // Continue draining to prevent blocking the child process
                    } else if !truncated {
                        if !stderr_output.is_empty() {
                            stderr_output.push('\n');
                        }
                        stderr_output.push_str(line.as_ref());
                    }
                }
                Err(e) => {
                    // Log the read error but continue - this preserves diagnostic info
                    eprintln!("Warning: Failed to read from stderr: {}", e);
                    break;
                }
            }
        }

        stderr_output
    });

    // Read stdout line by line with length limit
    let mut reader = BufReader::new(stdout);
    let mut parse_error_seen = false;
    let mut line_buf = Vec::new();
    let mut all_messages_size: usize = 0;

    loop {
        line_buf.clear();
        match read_line_with_limit(&mut reader, &mut line_buf, MAX_LINE_LENGTH).await {
            Ok(read_result) => {
                if read_result.bytes_read == 0 {
                    break; // EOF
                }

                // Check for line truncation - short-circuit to error instead of attempting parse
                if read_result.truncated {
                    let error_msg = format!(
                        "Output line exceeded {} byte limit and was truncated, cannot parse JSON.",
                        MAX_LINE_LENGTH
                    );
                    result.success = false;
                    result.error = Some(error_msg);
                    if !parse_error_seen {
                        parse_error_seen = true;
                        // Stop the child so it cannot block on a full pipe, then keep draining
                        let _ = child.start_kill();
                    }
                    continue;
                }

                // Convert to string
                let line = String::from_utf8_lossy(&line_buf);
                let line = line.trim_end_matches('\n').trim_end_matches('\r');

                if line.is_empty() {
                    continue;
                }

                // After a parse error, keep draining stdout to avoid blocking the child process
                if parse_error_seen {
                    continue;
                }

                // Parse JSON line
                let line_data: Value = match serde_json::from_str(line) {
                    Ok(data) => data,
                    Err(e) => {
                        record_parse_error(&mut result, &e, line);
                        if !parse_error_seen {
                            parse_error_seen = true;
                            // Stop the child so it cannot block on a full pipe, then keep draining
                            let _ = child.start_kill();
                        }
                        continue;
                    }
                };

                // Collect all messages with bounds checking
                if let Ok(map) =
                    serde_json::from_value::<HashMap<String, Value>>(line_data.clone())
                {
                    // Estimate size of this message (JSON serialized size)
                    let message_size =
                        serde_json::to_string(&map).map(|s| s.len()).unwrap_or(0);

                    // Check if adding this message would exceed byte limit
                    if all_messages_size + message_size <= MAX_ALL_MESSAGES_SIZE {
                        all_messages_size += message_size;
                        result.all_messages.push(map);
                    } else if !result.all_messages_truncated {
                        result.all_messages_truncated = true;
                    }
                }

                // Extract session_id from any event that includes it
                if let Some(session_id) = line_data.get("session_id").and_then(|v| v.as_str()) {
                    if !session_id.is_empty() {
                        result.session_id = session_id.to_string();
                    }
                }

                // Extract assistant text from Claude stream-json output.
                // We primarily look at `type == "assistant"` events and pull
                // text blocks from `message.content[*].text`. As a fallback,
                // we also consider `type == "result"` lines with a string
                // `result` field.
                if let Some(line_type) = line_data.get("type").and_then(|v| v.as_str()) {
                    match line_type {
                        "assistant" => {
                            if let Some(message) =
                                line_data.get("message").and_then(|v| v.as_object())
                            {
                                if let Some(content) =
                                    message.get("content").and_then(|v| v.as_array())
                                {
                                    for block in content {
                                        if block.get("type").and_then(|v| v.as_str())
                                            == Some("text")
                                        {
                                            if let Some(text) =
                                                block.get("text").and_then(|v| v.as_str())
                                            {
                                                let new_size =
                                                    result.agent_messages.len() + text.len();
                                                if new_size > MAX_AGENT_MESSAGES_SIZE {
                                                    if !result.agent_messages_truncated {
                                                        result.agent_messages.push_str(
                                                            "\n[... Agent messages truncated due to size limit ...]",
                                                        );
                                                        result.agent_messages_truncated = true;
                                                    }
                                                } else if !result.agent_messages_truncated {
                                                    if !result.agent_messages.is_empty()
                                                        && !text.is_empty()
                                                    {
                                                        result.agent_messages.push('\n');
                                                    }
                                                    result.agent_messages.push_str(text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "result" => {
                            if let Some(result_text) =
                                line_data.get("result").and_then(|v| v.as_str())
                            {
                                let new_size =
                                    result.agent_messages.len() + result_text.len();
                                if new_size > MAX_AGENT_MESSAGES_SIZE {
                                    if !result.agent_messages_truncated {
                                        result.agent_messages.push_str(
                                            "\n[... Agent messages truncated due to size limit ...]",
                                        );
                                        result.agent_messages_truncated = true;
                                    }
                                } else if !result.agent_messages_truncated {
                                    if !result.agent_messages.is_empty()
                                        && !result_text.is_empty()
                                    {
                                        result.agent_messages.push('\n');
                                    }
                                    result.agent_messages.push_str(result_text);
                                }
                            }

                            // If this result represents an error (`is_error: true`),
                            // surface it as a failure.
                            if line_data
                                .get("is_error")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                            {
                                result.success = false;
                                if let Some(result_text) =
                                    line_data.get("result").and_then(|v| v.as_str())
                                {
                                    result.error = Some(format!("Claude error: {}", result_text));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                // Create a simple IO error for the parse error
                let io_error = std::io::Error::from(e.kind());
                record_parse_error(&mut result, &serde_json::Error::io(io_error), "");
                break;
            }
        }
    }

    // Wait for process to finish
    let status = child
        .wait()
        .await
        .context("Failed to wait for claude command")?;

    // Collect stderr output with better error handling
    let stderr_output = match stderr_handle.await {
        Ok(output) => output,
        Err(e) => {
            // Log the join error but continue processing
            eprintln!("Warning: Failed to join stderr task: {}", e);
            String::new()
        }
    };

    if !status.success() {
        result.success = false;
        let error_msg = if let Some(ref err) = result.error {
            err.clone()
        } else {
            format!("claude command failed with exit code: {:?}", status.code())
        };

        // Append stderr diagnostics if available
        if !stderr_output.is_empty() {
            result.error = Some(format!("{}\nStderr: {}", error_msg, stderr_output));
        } else {
            result.error = Some(error_msg);
        }
    } else if !stderr_output.is_empty() {
        // On success, put stderr in warnings field instead of error
        result.warnings = Some(stderr_output);
    }

    Ok(enforce_required_fields(result, ValidationMode::Full))
}

fn record_parse_error(result: &mut ClaudeResult, error: &serde_json::Error, line: &str) {
    let parse_msg = format!("JSON parse error: {}. Line: {}", error, line);
    result.success = false;
    result.error = match result.error.take() {
        Some(existing) if !existing.is_empty() => Some(format!("{existing}\n{parse_msg}")),
        _ => Some(parse_msg),
    };
}

fn push_warning(existing: Option<String>, warning: &str) -> Option<String> {
    match existing {
        Some(mut current) => {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(warning);
            Some(current)
        }
        None => Some(warning.to_string()),
    }
}

fn enforce_required_fields(mut result: ClaudeResult, mode: ValidationMode) -> ClaudeResult {
    // Skip validation for cases where we already have a well-defined error (e.g., timeout, truncation)
    if mode == ValidationMode::Skip {
        return result;
    }

    // Skip session_id check if there's already an error (e.g., truncation, I/O error)
    // to avoid masking the original error
    if result.session_id.is_empty() && result.error.is_none() {
        result.success = false;
        result.error = Some("Failed to get SESSION_ID from the Claude session.".to_string());
    }

    if result.agent_messages.is_empty() {
        // Preserve success but surface as a warning so callers can decide how to handle it
        let warning_msg = "No agent_messages returned; check Claude CLI output or enable richer logging if needed.";
        result.warnings = push_warning(result.warnings.take(), warning_msg);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_creation() {
        let opts = Options {
            prompt: "test prompt".to_string(),
            working_dir: PathBuf::from("/tmp"),
            session_id: None,
            additional_args: Vec::new(),
            timeout_secs: None,
        };

        assert_eq!(opts.prompt, "test prompt");
        assert_eq!(opts.working_dir, PathBuf::from("/tmp"));
    }

    #[test]
    fn test_options_with_session() {
        let opts = Options {
            prompt: "resume task".to_string(),
            working_dir: PathBuf::from("/tmp"),
            session_id: Some("test-session-123".to_string()),
            additional_args: vec!["--json".to_string()],
            timeout_secs: Some(600),
        };

        assert_eq!(opts.session_id, Some("test-session-123".to_string()));
        assert_eq!(opts.timeout_secs, Some(600));
    }

    #[test]
    fn test_record_parse_error_sets_failure_and_appends_message() {
        let mut result = ClaudeResult {
            success: true,
            session_id: "session".to_string(),
            agent_messages: "ok".to_string(),
            agent_messages_truncated: false,
            all_messages: Vec::new(),
            all_messages_truncated: false,
            error: Some("existing".to_string()),
            warnings: None,
        };

        let err = serde_json::from_str::<Value>("not-json").unwrap_err();
        record_parse_error(&mut result, &err, "not-json");

        assert!(!result.success);
        assert!(result.error.as_ref().unwrap().contains("JSON parse error"));
        assert!(result.error.as_ref().unwrap().contains("existing"));
    }

    #[test]
    fn test_enforce_required_fields_warns_on_missing_agent_messages() {
        let result = ClaudeResult {
            success: true,
            session_id: "session".to_string(),
            agent_messages: String::new(),
            agent_messages_truncated: false,
            all_messages: vec![HashMap::new()],
            all_messages_truncated: false,
            error: None,
            warnings: None,
        };

        let updated = enforce_required_fields(result, ValidationMode::Full);

        assert!(updated.success);
        assert!(updated
            .warnings
            .as_ref()
            .unwrap()
            .contains("No agent_messages"));
    }

    #[test]
    fn test_enforce_required_fields_requires_session_id() {
        let result = ClaudeResult {
            success: true,
            session_id: String::new(),
            agent_messages: "msg".to_string(),
            agent_messages_truncated: false,
            all_messages: Vec::new(),
            all_messages_truncated: false,
            error: None,
            warnings: None,
        };

        let updated = enforce_required_fields(result, ValidationMode::Full);

        assert!(!updated.success);
        assert!(updated
            .error
            .as_ref()
            .unwrap()
            .contains("Failed to get SESSION_ID"));
    }

    #[test]
    fn test_push_warning_appends_with_newline() {
        let combined = push_warning(Some("first".to_string()), "second").unwrap();
        assert!(combined.contains("first"));
        assert!(combined.contains("second"));
        assert!(combined.contains('\n'));
    }

    #[test]
    fn test_enforce_required_fields_skips_validation_when_requested() {
        // Simulate a timeout result with empty session_id and agent_messages
        let result = ClaudeResult {
            success: false,
            session_id: String::new(),
            agent_messages: String::new(),
            agent_messages_truncated: false,
            all_messages: Vec::new(),
            all_messages_truncated: false,
            error: Some("Claude execution timed out after 10 seconds".to_string()),
            warnings: None,
        };

        let updated = enforce_required_fields(result, ValidationMode::Skip);

        // When skipping validation, the original error should be preserved
        assert!(!updated.success);
        assert_eq!(
            updated.error.unwrap(),
            "Claude execution timed out after 10 seconds"
        );
        // Should NOT have session_id error appended
        // Should NOT have agent_messages warning
        assert!(updated.warnings.is_none());
        assert!(updated.session_id.is_empty());
    }

    #[test]
    fn test_enforce_required_fields_skips_session_id_when_error_exists() {
        // Simulate a truncation error with empty session_id
        let result = ClaudeResult {
            success: false,
            session_id: String::new(),
            agent_messages: String::new(),
            agent_messages_truncated: false,
            all_messages: Vec::new(),
            all_messages_truncated: false,
            error: Some(
                "Output line exceeded 1048576 byte limit and was truncated, cannot parse JSON."
                    .to_string(),
            ),
            warnings: None,
        };

        let updated = enforce_required_fields(result, ValidationMode::Full);

        // When there's already an error, session_id check should be skipped
        assert!(!updated.success);
        let error = updated.error.unwrap();
        assert!(error.contains("truncated"));
        assert!(
            !error.contains("SESSION_ID"),
            "Should not add session_id error when truncation error exists"
        );
        // Agent_messages warning should still be added since it's a separate concern
        assert!(updated.warnings.is_some());
        assert!(updated.warnings.unwrap().contains("No agent_messages"));
    }
}
