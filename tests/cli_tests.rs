mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn cli_no_args_shows_usage() {
    let temp = setup_temp_dir();

    let result = run_command(&[], &temp);

    assert!(!result.success, "Should fail when no command provided");
    assert!(result.stderr.contains("Usage: knecht"),
        "Should show usage message, got: {}", result.stderr);

    cleanup_temp_dir(temp);
}

#[test]
fn cli_unknown_command_fails() {
    let temp = setup_temp_dir();

    let result = run_command(&["nonexistent"], &temp);

    assert!(!result.success, "Should fail for unknown command");
    assert!(result.stderr.contains("unrecognized subcommand") || result.stderr.contains("nonexistent"),
        "Should show unknown command error, got: {}", result.stderr);

    cleanup_temp_dir(temp);
}

#[test]
fn help_flag_shows_usage() {
    let temp = setup_temp_dir();

    // --help should show usage information and succeed
    let result = run_command(&["--help"], &temp);
    assert!(result.success, "--help should succeed, got stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("Usage:") || result.stdout.to_lowercase().contains("usage"),
        "--help should contain 'Usage', got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}

#[test]
fn subcommand_help_shows_usage() {
    let temp = setup_temp_dir();

    // help for subcommand should show its usage
    let result = run_command(&["add", "--help"], &temp);
    assert!(result.success, "add --help should succeed, got stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("add") || result.stdout.contains("Add"),
        "add --help should mention 'add', got: {}",
        result.stdout
    );

    cleanup_temp_dir(temp);
}
