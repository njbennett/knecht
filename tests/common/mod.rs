#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct TestResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn setup_temp_dir() -> PathBuf {
    let temp = std::env::temp_dir().join(format!("knecht-test-{}", rand_string()));
    fs::create_dir_all(&temp).unwrap();
    temp
}

fn rand_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let thread_id = std::thread::current().id();
    format!("{}-{:?}", nanos, thread_id)
}

pub fn cleanup_temp_dir(path: PathBuf) {
    let _ = fs::remove_dir_all(path);
}

pub fn run_command(args: &[&str], working_dir: &PathBuf) -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_knecht"))
        .args(args)
        .current_dir(working_dir)
        .output()
        .expect("Failed to execute command");

    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

pub fn with_initialized_repo<F>(test_fn: F)
where
    F: FnOnce(&PathBuf),
{
    let temp = setup_temp_dir();
    let init_result = run_command(&["init"], &temp);
    assert!(
        init_result.success,
        "init command failed: {}",
        init_result.stderr
    );

    test_fn(&temp);

    cleanup_temp_dir(temp);
}

/// Extracts the task ID from command output like "Created task-abc123"
pub fn extract_task_id(output: &str) -> String {
    output
        .lines()
        .find(|l| l.contains("task-"))
        .and_then(|l| l.split("task-").nth(1))
        .map(|s| s.split_whitespace().next().unwrap_or(""))
        .unwrap_or("")
        .to_string()
}
