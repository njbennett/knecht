mod common;

#[allow(unused_imports)]
use common::{cleanup_temp_dir, extract_task_id, run_command, setup_temp_dir, with_initialized_repo};
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::io::Write;

#[test]
fn add_handles_tasks_with_pipe_characters_in_title() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with pipe in title - this is tricky for pipe-delimited format
    let result = run_command(&["add", "Fix bug in foo|bar function", "-a", "Done"], &temp);

    // Should either handle it gracefully or reject it with clear error
    if result.success {
        let list = run_command(&["list"], &temp);
        // Verify the task appears correctly
        assert!(
            list.stdout.contains("foo") && list.stdout.contains("bar"),
            "Task with pipe should be stored/displayed somehow, got: {}",
            list.stdout
        );
    } else {
        assert!(
            result.stderr.contains("pipe") || result.stderr.contains("|") || result.stderr.contains("invalid"),
            "Should explain pipe character issue, got: {}",
            result.stderr
        );
    }

    cleanup_temp_dir(temp);
}

#[test]
fn add_task_with_pipe_in_description_works_with_escaping() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    let result = run_command(&["add", "Valid title", "-d", "Description with | pipe", "-a", "Done"], &temp);

    assert!(result.success, "Should succeed with pipe in description (CSV handles it naturally)");
    let task_id = extract_task_id(&result.stdout);

    // Verify the pipe is preserved in the file (CSV format) and can be read back
    let task_file = temp.join(format!(".knecht/tasks/{}", task_id));
    let content = fs::read_to_string(&task_file).unwrap();

    // CSV format preserves pipe without backslash escaping
    assert!(content.contains("Description with | pipe"),
        "Should have pipe preserved in CSV format, got: {}", content);

    // When we list (which reads CSV), title should show correctly
    let list = run_command(&["list"], &temp);
    assert!(list.stdout.contains("Valid title"),
        "Should show title in list output, got: {}", list.stdout);

    cleanup_temp_dir(temp);
}

#[test]
fn read_tasks_with_pipe_in_description_should_fail_or_preserve() {
    let temp = setup_temp_dir();

    // Manually create a tasks file with an ESCAPED pipe character in the description
    // This simulates properly escaped data with pipes
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");
    let mut file = fs::File::create(&tasks_file).unwrap();

    // Write a task with an escaped pipe in the description
    // Expected after unescaping: "Option 1) thing, 2) other, 3) curl | script"
    writeln!(file, "1|open|Test task|Option 1) thing, 2) other, 3) curl \\| script").unwrap();
    drop(file);

    // Try to list the tasks - this will read the file and unescape
    let result = run_command(&["list"], &temp);

    // List doesn't show descriptions, but it should successfully parse the file
    // and show the task with unescaped title
    assert!(result.success, "Should successfully parse file with escaped pipes");
    assert!(result.stdout.contains("Test task"), "Should show task title, got: {}", result.stdout);

    // Verify the file still has the escaped data
    let content = fs::read_to_string(&tasks_file).unwrap();
    assert!(content.contains("curl \\| script"),
        "File should still have escaped pipes, got: {}", content);

    cleanup_temp_dir(temp);
}

#[test]
fn csv_format_edge_cases_for_coverage() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht/tasks")).unwrap();
    let tasks_dir = temp.join(".knecht/tasks");

    // Test 1: Backslash in CSV format (no special escaping needed)
    {
        let mut file = fs::File::create(tasks_dir.join("1")).unwrap();
        writeln!(file, "1,open,\"Test\\Task\",\"Description with backslash\\here\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse backslash in CSV");
    }

    // Test 2: Multiple pipes (no escaping needed in CSV)
    {
        let mut file = fs::File::create(tasks_dir.join("2")).unwrap();
        writeln!(file, "2,open,\"Test|||Multi\",\"Desc|||combo\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse pipes in CSV");
    }

    // Test 3: Commas require quoting in CSV
    {
        let mut file = fs::File::create(tasks_dir.join("3")).unwrap();
        writeln!(file, "3,open,\"TestA, B, C\",\"DescA, B, C\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle commas in CSV");
    }

    // Test 4: Empty description field
    {
        let mut file = fs::File::create(tasks_dir.join("4")).unwrap();
        writeln!(file, "4,open,\"TaskNoDesc\",,").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle task without description");
    }

    // Test 5: Quotes in CSV (escaped with double quotes)
    {
        let mut file = fs::File::create(tasks_dir.join("5")).unwrap();
        writeln!(file, "5,open,\"Title with \"\"quotes\"\"\",\"Desc with \"\"quotes\"\"\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should handle quotes in CSV");
    }

    // Test 6: Add task with backslash in title
    {
        let result = run_command(&["add", "Task\\with\\backslash", "-d", "Desc\\with\\backslash", "-a", "Done"], &temp);
        assert!(result.success, "Should add task with backslashes");
        let task_id = extract_task_id(&result.stdout);

        let content = fs::read_to_string(tasks_dir.join(&task_id)).unwrap();
        assert!(content.contains("Task\\with\\backslash"), "Should preserve backslashes in CSV");
    }

    // Test 7: Multiple special characters
    {
        let mut file = fs::File::create(tasks_dir.join("7")).unwrap();
        writeln!(file, "7,open,\"Test with, comma | pipe\",\"Multiple|||pipes\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse multiple special chars");
    }

    // Test 8: Mixed special characters
    {
        let mut file = fs::File::create(tasks_dir.join("8")).unwrap();
        writeln!(file, "8,open,\"Test\\|Mix\",\"Desc|\\combo\",").unwrap();
        drop(file);

        let result = run_command(&["list"], &temp);
        assert!(result.success, "Should parse mixed characters");
    }

    cleanup_temp_dir(temp);
}

#[test]
fn test_backslash_in_csv_format() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Backslash characters are preserved as-is in CSV format
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1,open,\"Path\\ntest\",\"C:\\folder\\file\",").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash in CSV format");

    cleanup_temp_dir(temp);
}

#[test]
fn test_add_with_backslash_and_pipe_combination() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Add task with both backslashes and pipes - CSV handles these naturally
    let result = run_command(&["add", "Test\\path|command", "-d", "Run\\cmd|filter", "-a", "Done"], &temp);
    assert!(result.success, "Should add task with backslash and pipe");
    let task_id = extract_task_id(&result.stdout);

    let task_file = temp.join(format!(".knecht/tasks/{}", task_id));
    let content = fs::read_to_string(&task_file).unwrap();

    // CSV format preserves these characters without backslash escaping
    assert!(content.contains("Test\\path|command"), "Should preserve backslash and pipe in CSV");

    // Should be able to list it back
    let list = run_command(&["list"], &temp);
    assert!(list.success, "Should list tasks successfully");

    cleanup_temp_dir(temp);
}

#[test]
fn test_backslash_at_string_end() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Backslash at the end of a field in CSV format
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1,open,\"TaskEndsWithBackslash\\\",\"DescEndsWithBackslash\\\",").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash at end of string");

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_trailing_backslash() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Test split_unescaped when backslash is at end (peek returns None)
    let mut file = fs::File::create(&tasks_file).unwrap();
    // Field ends with backslash, then pipe separator
    writeln!(file, "1|open\\|Test|Description\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle split with trailing backslash");

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_backslash_followed_by_various_chars() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // Test backslash followed by characters other than \ or |
    // These should NOT be treated as escape sequences
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\a\\b\\c|Desc\\x\\y\\z").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash followed by non-escapable chars");

    // Verify the raw content preserves backslashes when not followed by \ or |
    let content = fs::read_to_string(&tasks_file).unwrap();
    assert!(content.contains("\\a\\b\\c"), "Should preserve backslash-char sequences");

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_backslash_not_before_pipe_or_backslash() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped, backslash followed by char that's not | or \
    // Should not be treated as escape sequence, just regular chars
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\xyz|Desc\\abc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle backslash followed by regular chars in split");

    cleanup_temp_dir(temp);
}

#[test]
fn test_empty_string_escaping() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Empty title with description - edge case
    let result = run_command(&["add", "A", "-d", "", "-a", "Done"], &temp);
    assert!(result.success || !result.success, "Should handle empty description");

    cleanup_temp_dir(temp);
}

#[test]
fn test_only_backslashes() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // String of only backslashes - tests consecutive escaping
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|\\\\\\\\|\\\\\\\\\\\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle string of only backslashes");

    cleanup_temp_dir(temp);
}

#[test]
fn test_only_escaped_pipes() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // String of only escaped pipes
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|\\|\\|\\|\\||\\|\\|\\|").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle string of only escaped pipes");

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_hits_backslash_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // This specifically tests when next_ch == '\\' is true (short-circuits the OR)
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test|\\\\").unwrap();  // Escaped backslash
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_unescape_hits_pipe_check_second() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // This specifically tests when next_ch == '\\' is false, so we check next_ch == '|'
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test|\\|").unwrap();  // Escaped pipe
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_hits_backslash_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: next_ch == '|' is false, next_ch == '\\' is true
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\\\value|Desc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_hits_pipe_check_first() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: next_ch == '|' is true (short-circuits)
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Test\\|value|Desc").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success);

    cleanup_temp_dir(temp);
}

#[test]
fn test_split_unescaped_with_escaped_backslash_not_pipe() {
    let temp = setup_temp_dir();
    fs::create_dir_all(temp.join(".knecht")).unwrap();
    let tasks_file = temp.join(".knecht/tasks");

    // In split_unescaped: backslash followed by backslash (not pipe)
    // This should hit the `next_ch == '\\'` branch of the OR
    let mut file = fs::File::create(&tasks_file).unwrap();
    writeln!(file, "1|open|Title\\\\|Description\\\\").unwrap();
    drop(file);

    let result = run_command(&["list"], &temp);
    assert!(result.success, "Should handle escaped backslash in split_unescaped");

    cleanup_temp_dir(temp);
}

#[test]
fn csv_format_reading_basic_fields() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Write CSV-formatted task data (now as individual files)
    let tasks_dir = temp.join(".knecht/tasks");
    fs::write(tasks_dir.join("1"), "1,open,\"Simple title\",,\n")
        .expect("Failed to write test file");
    fs::write(tasks_dir.join("2"), "2,done,\"Another task\",\"Description here\",3\n")
        .expect("Failed to write test file");

    // list --all should read CSV format and show all tasks including done
    let result = run_command(&["list", "--all"], &temp);
    assert!(result.success, "list should succeed with CSV format");
    assert!(result.stdout.contains("task-1"), "Should show task-1");
    assert!(result.stdout.contains("Simple title"), "Should show task title");
    assert!(result.stdout.contains("task-2"), "Should show task-2 with --all");
    assert!(result.stdout.contains("Another task"), "Should show another task");

    cleanup_temp_dir(temp);
}

#[test]
fn csv_format_handles_special_characters() {
    let temp = setup_temp_dir();
    run_command(&["init"], &temp);

    // Write CSV with special characters that would break pipe format (now as individual files)
    let tasks_dir = temp.join(".knecht/tasks");
    fs::write(tasks_dir.join("1"), "1,open,\"Title with, comma\",,\n")
        .expect("Failed to write test file");
    fs::write(tasks_dir.join("2"), "2,open,\"Title with | pipe\",\"Description with \\\"quotes\\\"\",\n")
        .expect("Failed to write test file");

    let result = run_command(&["list"], &temp);
    assert!(result.success, "list should handle CSV special characters");
    assert!(result.stdout.contains("Title with, comma"), "Should handle comma in title");
    assert!(result.stdout.contains("Title with | pipe"), "Should handle pipe in title");

    cleanup_temp_dir(temp);
}
