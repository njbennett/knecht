mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;

#[test]
fn next_suggests_task_with_highest_pain_count() {
    with_initialized_repo(|temp| {
        // Add tasks
        run_command(&["add", "Low priority task"], &temp);
        let r2 = run_command(&["add", "Medium pain task"], &temp);
        let r3 = run_command(&["add", "High pain task"], &temp);
        run_command(&["add", "Another low priority"], &temp);
        let r5 = run_command(&["add", "Medium pain again"], &temp);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);
        let id5 = extract_task_id(&r5.stdout);

        // Set pain counts using pain command
        run_command(&["pain", "-t", &format!("task-{}", id2), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id2), "-d", "Pain 2"], &temp); // pain count: 2

        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 2"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 3"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 4"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 5"], &temp); // pain count: 5

        run_command(&["pain", "-t", &format!("task-{}", id5), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id5), "-d", "Pain 2"], &temp); // pain count: 2

        // Run 'knecht next'
        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("High pain task"),
            "Should suggest task with highest pain count, got: {}",
            result.stdout
        );
        assert!(
            result.stdout.contains("pain count: 5"),
            "Should mention the pain count, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_prefers_older_task_when_pain_counts_equal() {
    with_initialized_repo(|temp| {
        // Add tasks
        let r1 = run_command(&["add", "First task"], &temp);
        let r2 = run_command(&["add", "Second task"], &temp);
        let r3 = run_command(&["add", "Third task"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);
        let id3 = extract_task_id(&r3.stdout);

        // Set same pain count on all tasks
        for i in 0..3 {
            run_command(&["pain", "-t", &format!("task-{}", id1), "-d", &format!("Pain {}", i)], &temp);
            run_command(&["pain", "-t", &format!("task-{}", id2), "-d", &format!("Pain {}", i)], &temp);
            run_command(&["pain", "-t", &format!("task-{}", id3), "-d", &format!("Pain {}", i)], &temp);
        }

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        // Find which ID is lexicographically smallest (that's the tiebreaker)
        let ids = [&id1, &id2, &id3];
        let smallest_id = ids.iter().min().unwrap();
        assert!(
            result.stdout.contains(&format!("task-{}", smallest_id)),
            "Should suggest task with lexicographically smallest ID when pain counts equal, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_skips_done_tasks() {
    with_initialized_repo(|temp| {
        // Add tasks
        let r1 = run_command(&["add", "High pain but done"], &temp);
        let r2 = run_command(&["add", "Lower pain but open"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Set pain counts
        for i in 0..5 {
            run_command(&["pain", "-t", &format!("task-{}", id1), "-d", &format!("Pain {}", i)], &temp);
        }
        for i in 0..2 {
            run_command(&["pain", "-t", &format!("task-{}", id2), "-d", &format!("Pain {}", i)], &temp);
        }

        // Mark first task as done
        run_command(&["done", &format!("task-{}", id1)], &temp);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains(&format!("task-{}", id2)),
            "Should skip done tasks and suggest task-{}, got: {}",
            id2, result.stdout
        );
    });
}

#[test]
fn next_handles_no_open_tasks() {
    with_initialized_repo(|temp| {
        // Add and complete a task
        let add_result = run_command(&["add", "Only task"], &temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["done", &format!("task-{}", task_id)], &temp);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains("No open tasks") || result.stdout.contains("no open tasks"),
            "Should indicate no open tasks available, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_fails_gracefully_when_tasks_file_unreadable() {
    with_initialized_repo(|temp| {
        // Add a task
        run_command(&["add", "Some task"], &temp);
        
        // Make tasks file unreadable
        let tasks_file = temp.join(".knecht/tasks");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&tasks_file, perms).unwrap();
        }
        
        let result = run_command(&["next"], &temp);
        
        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tasks_file).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&tasks_file, perms).unwrap();
        }
        
        assert!(!result.success, "next should fail when tasks file is unreadable");
        assert!(
            result.stderr.contains("Error reading tasks") || result.stderr.contains("error"),
            "should indicate error reading tasks, got: {}",
            result.stderr
        );
    });
}

#[test]
fn next_displays_task_with_description() {
    with_initialized_repo(|temp| {
        // Add a task with description
        let add_result = run_command(&["add", "Important task", "-d", "This task has a detailed description explaining what needs to be done"], &temp);
        let task_id = extract_task_id(&add_result.stdout);

        // Add pain to make it more likely to be selected
        run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", task_id), "-d", "Pain 2"], &temp);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(result.stdout.contains(&format!("task-{}", task_id)), "Should suggest the task");
        assert!(result.stdout.contains("Important task"), "Should show title");
        assert!(
            result.stdout.contains("This task has a detailed description"),
            "Should show description, got: {}",
            result.stdout
        );
        assert!(result.stdout.contains("pain count: 2"), "Should show pain count");
    });
}

#[test]
fn next_with_zero_pain_count() {
    with_initialized_repo(|temp| {
        // Add tasks - one will have pain_count 0 (no pain added), one will be without pain_count
        let r1 = run_command(&["add", "Task with no pain"], &temp);
        let r2 = run_command(&["add", "Another task"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        // Should suggest task with smaller ID (tiebreaker when both have no pain)
        let smaller_id = if id1 < id2 { &id1 } else { &id2 };
        assert!(result.stdout.contains(&format!("task-{}", smaller_id)), "Should suggest task with smaller ID");
        // Should not show pain count line when pain is 0 or None
        assert!(
            !result.stdout.contains("pain count:"),
            "Should not show pain count for tasks with 0 or no pain, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_prefers_unblocked_subtasks_over_parent_task() {
    with_initialized_repo(|temp| {
        // Create a parent task with high pain count
        let r1 = run_command(&["add", "Large feature with verification", "-d", "This is a big task"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        for i in 0..3 {
            run_command(&["pain", "-t", &format!("task-{}", id1), "-d", &format!("Pain {}", i)], &temp);
        }

        // Create subtasks that block the parent task
        let r2 = run_command(&["add", "Foundation work", "-d", "Must be done first"], &temp);
        let id2 = extract_task_id(&r2.stdout);
        let r3 = run_command(&["add", "Build feature A", "-d", "Needs foundation"], &temp);
        let id3 = extract_task_id(&r3.stdout);
        let r4 = run_command(&["add", "Build feature B", "-d", "Needs foundation"], &temp);
        let id4 = extract_task_id(&r4.stdout);

        // Parent is blocked by all subtasks (can't complete parent until subtasks done)
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id3)], &temp);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id4)], &temp);

        // Feature tasks also blocked by foundation
        run_command(&["block", &format!("task-{}", id3), "by", &format!("task-{}", id2)], &temp);
        run_command(&["block", &format!("task-{}", id4), "by", &format!("task-{}", id2)], &temp);

        // Complete the foundation
        run_command(&["done", &format!("task-{}", id2)], &temp);

        // Now next should suggest id3 or id4 (unblocked subtasks) instead of id1 (parent)
        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        // Should NOT suggest id1 (the parent with high pain) because it has open subtasks
        assert!(
            !result.stdout.contains(&format!("task-{}", id1)),
            "Should not suggest parent task when it has unblocked subtasks, got: {}",
            result.stdout
        );
        // Should suggest one of the unblocked subtasks (id3 or id4)
        assert!(
            result.stdout.contains(&format!("task-{}", id3)) || result.stdout.contains(&format!("task-{}", id4)),
            "Should suggest one of the unblocked subtasks, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_handles_three_level_blocker_tree() {
    with_initialized_repo(|temp| {
        // Create a three-level blocker tree like task-143 → task-176 → tasks 184-192
        // Root task with high pain count
        let r1 = run_command(&["add", "Root feature", "-d", "Top level feature"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        for i in 0..3 {
            run_command(&["pain", "-t", &format!("task-{}", id1), "-d", &format!("Pain {}", i)], &temp);
        }

        // Middle task (blocks root)
        let r2 = run_command(&["add", "Middle task", "-d", "Intermediate step"], &temp);
        let id2 = extract_task_id(&r2.stdout);
        run_command(&["block", &format!("task-{}", id1), "by", &format!("task-{}", id2)], &temp);

        // Leaf tasks (block middle task)
        let r3 = run_command(&["add", "Leaf task A", "-d", "First leaf"], &temp);
        let id3 = extract_task_id(&r3.stdout);
        let r4 = run_command(&["add", "Leaf task B", "-d", "Second leaf"], &temp);
        let id4 = extract_task_id(&r4.stdout);
        let r5 = run_command(&["add", "Leaf task C", "-d", "Third leaf"], &temp);
        let id5 = extract_task_id(&r5.stdout);
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id3)], &temp);
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id4)], &temp);
        run_command(&["block", &format!("task-{}", id2), "by", &format!("task-{}", id5)], &temp);

        // Now next should suggest one of the leaf tasks (id3, id4, or id5)
        // NOT the root (id1) or middle (id2)
        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            !result.stdout.contains(&format!("task-{}", id1)),
            "Should not suggest root task when it has blockers, got: {}",
            result.stdout
        );
        assert!(
            !result.stdout.contains(&format!("task-{}", id2)),
            "Should not suggest middle task when it has blockers, got: {}",
            result.stdout
        );
        // Should suggest one of the leaf tasks
        assert!(
            result.stdout.contains(&format!("task-{}", id3)) ||
            result.stdout.contains(&format!("task-{}", id4)) ||
            result.stdout.contains(&format!("task-{}", id5)),
            "Should suggest one of the leaf tasks, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_prioritizes_delivered_tasks_over_open_tasks() {
    with_initialized_repo(|temp| {
        // Add several open tasks with varying pain counts
        let r1 = run_command(&["add", "High pain open task"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        run_command(&["pain", "-t", &format!("task-{}", id1), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id1), "-d", "Pain 2"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id1), "-d", "Pain 3"], &temp); // pain count: 3

        let r2 = run_command(&["add", "Low pain delivered task"], &temp);
        let id2 = extract_task_id(&r2.stdout);
        run_command(&["deliver", &format!("task-{}", id2)], &temp); // delivered with no pain

        let r3 = run_command(&["add", "Medium pain open task"], &temp);
        let id3 = extract_task_id(&r3.stdout);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 1"], &temp);
        run_command(&["pain", "-t", &format!("task-{}", id3), "-d", "Pain 2"], &temp); // pain count: 2

        // Even though task id1 has higher pain count, next should suggest id2
        // because delivered tasks take priority over open tasks
        let result = run_command(&["next"], &temp);

        assert!(result.success, "next command should succeed");
        assert!(
            result.stdout.contains(&format!("task-{}", id2)),
            "Should suggest delivered task over higher-pain open tasks, got: {}",
            result.stdout
        );
        assert!(
            result.stdout.contains("Low pain delivered task"),
            "Should show the delivered task title, got: {}",
            result.stdout
        );
    });
}

#[test]
fn next_skips_claimed_tasks() {
    // claimed tasks should be skipped by knecht next, just like done tasks
    with_initialized_repo(|temp| {
        // Add two tasks
        let r1 = run_command(&["add", "First task"], &temp);
        let r2 = run_command(&["add", "Second task"], &temp);
        let id1 = extract_task_id(&r1.stdout);
        let id2 = extract_task_id(&r2.stdout);

        // Claim the first task
        run_command(&["start", &format!("task-{}", id1)], &temp);

        // next should suggest the second task, not the first
        let result = run_command(&["next"], &temp);
        assert!(result.success, "next should succeed: {}", result.stderr);
        assert!(result.stdout.contains(&format!("task-{}", id2)),
            "next should skip claimed task and suggest task-{}, got: {}", id2, result.stdout);
        assert!(!result.stdout.contains(&format!("task-{}", id1)),
            "next should not suggest claimed task-{}, got: {}", id1, result.stdout);
    });
}

#[test]
fn next_handles_all_tasks_claimed() {
    // When all tasks are claimed, next should indicate no available tasks
    with_initialized_repo(|temp| {
        // Add and claim all tasks
        let add_result = run_command(&["add", "Only task"], &temp);
        let task_id = extract_task_id(&add_result.stdout);
        run_command(&["start", &format!("task-{}", task_id)], &temp);

        let result = run_command(&["next"], &temp);
        assert!(result.success, "next should succeed: {}", result.stderr);
        assert!(result.stdout.contains("No open tasks") || result.stdout.contains("no open tasks"),
            "Should indicate no open tasks when all are claimed, got: {}", result.stdout);
    });
}
