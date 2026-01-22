mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn block_command_creates_blocker_relationship() {
    with_initialized_repo(|temp| {
        // Create two tasks
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create blocker: task-id1 is blocked by task-id2
        let result = run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        assert!(result.success, "block command should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Blocker added"), "Should confirm blocker added");
        assert!(result.stdout.contains(&format!("task-{}", id1)) && result.stdout.contains(&format!("task-{}", id2)),
                "Should mention both tasks");

        // Verify blockers file exists and contains the relationship
        let blockers_path = temp.join(".knecht/blockers");
        assert!(blockers_path.exists(), "blockers file should be created");

        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.contains(&format!("task-{}|task-{}", id1, id2)), "Should store blocker relationship");
    });
}

#[test]
fn block_command_fails_on_nonexistent_task() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        // Try to block nonexistent task
        let result = run_command(&["block", "task-999", "by", &format!("task-{}", id1)], &temp);
        assert!(!result.success, "block command should fail for nonexistent task");
        assert!(result.stderr.contains("does not exist") || result.stderr.contains("not found"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn block_command_fails_on_nonexistent_blocker() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        // Try to block by nonexistent task
        let result = run_command(&["block", &format!("task-{}", id1), "by", "task-999"], &temp);
        assert!(!result.success, "block command should fail for nonexistent blocker");
        assert!(result.stderr.contains("does not exist") || result.stderr.contains("not found"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn unblock_removes_blocker_relationship() {
    with_initialized_repo(|temp| {
        // Create tasks and blocker
        let r1 = run_command(&["add", "Blocked Task", "-a", "Can be started"], &temp);
        let r2 = run_command(&["add", "Blocker Task", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Remove blocker
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(result.success, "unblock command should succeed: {}", result.stderr);
        assert!(result.stdout.contains("Blocker removed"), "Should confirm removal");

        // Verify blockers file no longer contains the relationship
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(!content.contains(&format!("task-{}|task-{}", id1, id2)), "Should remove blocker relationship");

        // Start should now succeed
        let start_result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(start_result.success, "start should succeed after unblocking");
    });
}

#[test]
fn unblock_fails_when_relationship_does_not_exist() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Try to remove nonexistent blocker
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(!result.success, "unblock should fail when relationship doesn't exist");
        assert!(result.stderr.contains("not blocked") || result.stderr.contains("does not exist"),
                "Should have helpful error message: {}", result.stderr);
    });
}

#[test]
fn multiple_blockers_all_prevent_start() {
    with_initialized_repo(|temp| {
        // Create tasks (blocked task needs acceptance criteria so start fails due to blockers, not missing criteria)
        let r1 = run_command(&["add", "Blocked Task", "-a", "Can be started"], &temp);
        let r2 = run_command(&["add", "Blocker 1", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Blocker 2", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create multiple blockers
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);

        // Start should fail
        let result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(!result.success, "start should fail with multiple open blockers");
        assert!(result.stderr.contains(&format!("task-{}", id2)) && result.stderr.contains(&format!("task-{}", id3)),
                "Should list all blocking tasks: {}", result.stderr);
    });
}

#[test]
fn block_fails_when_blockers_file_cannot_be_written() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Make blockers file read-only
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, "").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&blockers_path, perms).unwrap();
        }

        #[cfg(windows)]
        {
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_readonly(true);
            fs::set_permissions(&blockers_path, perms).unwrap();
        }

        // Try to add blocker - should fail
        let result = run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        assert!(!result.success, "block should fail when file cannot be written");
        assert!(result.stderr.contains("Failed to write") || result.stderr.contains("Permission denied"),
                "Should have write error message: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_when_blockers_file_cannot_be_written() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Make blockers file read-only
        let blockers_path = temp.join(".knecht/blockers");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&blockers_path, perms).unwrap();
        }
        
        #[cfg(windows)]
        {
            let mut perms = fs::metadata(&blockers_path).unwrap().permissions();
            perms.set_readonly(true);
            fs::set_permissions(&blockers_path, perms).unwrap();
        }

        // Try to remove blocker - should fail
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(!result.success, "unblock should fail when file cannot be written");
        assert!(result.stderr.contains("Failed to write") || result.stderr.contains("Permission denied"),
                "Should have write error message: {}", result.stderr);
    });
}

#[test]
fn block_fails_with_malformed_command_no_by() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Try block without "by" keyword
        let result = run_command(&["block", &format!("task-{}", id1), &format!("task-{}", id2)], &temp);
        assert!(!result.success, "block should fail without 'by' keyword");
        // Clap shows "invalid value" and "possible values: by"
        assert!(result.stderr.contains("invalid value") || result.stderr.contains("possible values"),
            "Should show error about invalid value: {}", result.stderr);
    });
}

#[test]
fn block_fails_with_too_few_arguments() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        // Try block with only one argument
        let result = run_command(&["block", &format!("task-{}", id1)], &temp);
        assert!(!result.success, "block should fail with too few arguments");
        assert!(result.stderr.contains("Usage:"), "Should show usage: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_with_malformed_command_no_from() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Try unblock without "from" keyword
        let result = run_command(&["unblock", &format!("task-{}", id1), &format!("task-{}", id2)], &temp);
        assert!(!result.success, "unblock should fail without 'from' keyword");
        // Clap shows "invalid value" and "possible values: from"
        assert!(result.stderr.contains("invalid value") || result.stderr.contains("possible values"),
            "Should show error about invalid value: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_with_too_few_arguments() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);

        // Try unblock with only one argument
        let result = run_command(&["unblock", &format!("task-{}", id1)], &temp);
        assert!(!result.success, "unblock should fail with too few arguments");
        assert!(result.stderr.contains("Usage:"), "Should show usage: {}", result.stderr);
    });
}

#[test]
fn unblock_fails_when_blockers_file_does_not_exist() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Try to unblock without ever creating blockers file
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(!result.success, "unblock should fail when blockers file doesn't exist");
        assert!(result.stderr.contains("not blocked"), "Should say task is not blocked: {}", result.stderr);
    });
}

#[test]
fn unblock_preserves_file_format_when_removing_middle_blocker() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Task C", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create three blocker relationships
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id3)], &temp);
        
        // Remove middle one
        run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id3)], &temp);

        // Verify file still has proper format
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.contains(&format!("task-{}|task-{}", id1, id2)), "Should preserve first blocker");
        assert!(!content.contains(&format!("task-{}|task-{}", id1, id3)), "Should remove middle blocker");
        assert!(content.contains(&format!("task-{}|task-{}", id2, id3)), "Should preserve last blocker");
    });
}

#[test]
fn unblock_preserves_other_blockers_with_empty_lines() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Task C", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create blockers file with empty lines
        let blockers_path = temp.join(".knecht/blockers");
        fs::write(&blockers_path, format!("task-{}|task-{}\n\ntask-{}|task-{}\n", id1, id2, id1, id3)).unwrap();

        // Remove one blocker
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(result.success, "unblock should succeed");

        // Verify the other blocker is preserved
        let show_result = run_command(&["show", &format!("task-{}", id1)], &temp);
        assert!(show_result.stdout.contains(&format!("task-{}", id3)), "Should preserve other blocker");
        assert!(!show_result.stdout.contains(&format!("task-{}", id2)), "Should remove specified blocker");
    });
}

#[test]
fn unblock_fails_when_file_exists_but_relationship_not_found() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Done"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let r3 = run_command(&["add", "Task C", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Create a blocker file with a different relationship
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Try to remove a relationship that doesn't exist (but file does exist)
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id3)], &temp);
        assert!(!result.success, "unblock should fail when relationship doesn't exist in file");
        assert!(result.stderr.contains("not blocked"), "Should say task is not blocked: {}", result.stderr);
    });
}

#[test]
fn unblock_removes_last_blocker_leaving_empty_file() {
    with_initialized_repo(|temp| {
        let r1 = run_command(&["add", "Task A", "-a", "Can be started"], &temp);
        let r2 = run_command(&["add", "Task B", "-a", "Done"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Create single blocker
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Remove the only blocker
        let result = run_command(&["unblock", &format!("task-{}", id1), "from", &format!("task-{}", id2)], &temp);
        assert!(result.success, "unblock should succeed");

        // Verify file is empty
        let blockers_path = temp.join(".knecht/blockers");
        let content = fs::read_to_string(&blockers_path).unwrap();
        assert!(content.is_empty(), "blockers file should be empty");

        // Verify task can now be started
        let start_result = run_command(&["start", &format!("task-{}", id1)], &temp);
        assert!(start_result.success, "start should succeed after removing last blocker");
    });
}
