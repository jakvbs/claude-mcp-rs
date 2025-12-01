use claude_mcp_rs::claude::Options;
use std::path::PathBuf;

#[test]
fn test_options_validation() {
    // Test valid options
    let opts = Options {
        prompt: "Test prompt".to_string(),
        working_dir: PathBuf::from("/tmp"),
        session_id: None,
        additional_args: Vec::new(),
        timeout_secs: None,
    };

    assert!(!opts.prompt.is_empty());
    assert_eq!(opts.working_dir, PathBuf::from("/tmp"));
}

// Many of the older tests validating sandbox, model, images, and
// return_all_messages have been removed because those fields no
// longer exist on Options; sandboxing and other flags are configured
// via additional_args/global config instead.

#[test]
fn test_session_id_format() {
    let session_id = "550e8400-e29b-41d4-a716-446655440000";

    let opts = Options {
        prompt: "Continue task".to_string(),
        working_dir: PathBuf::from("/tmp"),
        session_id: Some(session_id.to_string()),
        additional_args: Vec::new(),
        timeout_secs: None,
    };

    assert!(opts.session_id.is_some());
    assert_eq!(opts.session_id.unwrap(), session_id);
}

#[test]
fn test_escape_prompt_integration() {
    // Removed since escape_prompt function was removed
    // Command::arg() handles platform-specific escaping automatically
    // This test is now empty as the functionality was removed
}

#[test]
fn test_working_directory_paths() {
    let paths = vec!["/tmp", "/home/user/project", ".", ".."];

    for path in paths {
        let opts = Options {
            prompt: "test".to_string(),
            working_dir: PathBuf::from(path),
            session_id: None,
            additional_args: Vec::new(),
            timeout_secs: None,
        };

        assert_eq!(opts.working_dir, PathBuf::from(path));
    }
}

// Model / profile / yolo-specific tests have been dropped since those
// concerns are now controlled via CLI flags in additional_args.
