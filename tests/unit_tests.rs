mod test_helpers;

use test_helpers::TestFileSystem;
use knecht::{read_tasks_with_fs, write_tasks_with_fs, get_next_id_with_fs, add_task_with_fs, mark_task_done_with_fs, find_task_by_id_with_fs, increment_pain_count_with_fs, find_next_task_with_fs, delete_task_with_fs, update_task_with_fs, Task, RealFileSystem, FileSystem};
use std::path::Path;

#[test]
fn test_read_tasks_error_on_open() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(read_tasks_with_fs(&fs).is_err());
}

#[test]
fn test_read_tasks_error_on_read_line() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("read");
    assert!(read_tasks_with_fs(&fs).is_err());
}

#[test]
fn test_write_tasks_error_on_create_dir() {
    let fs = TestFileSystem::new().fail("mkdir");
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None, pain_count: None }];
    assert!(write_tasks_with_fs(&tasks, &fs).is_err());
}

#[test]
fn test_write_tasks_error_on_create() {
    let fs = TestFileSystem::new().fail("create");
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None, pain_count: None }];
    assert!(write_tasks_with_fs(&tasks, &fs).is_err());
}

#[test]
fn test_write_tasks_error_on_write() {
    let fs = TestFileSystem::new().fail("write");
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None, pain_count: None }];
    assert!(write_tasks_with_fs(&tasks, &fs).is_err());
}

#[test]
fn test_get_next_id_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(get_next_id_with_fs(&fs).is_err());
}

#[test]
fn test_add_task_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(add_task_with_fs("New".to_string(), None, &fs).is_err());
}

#[test]
fn test_add_task_error_on_create_dir() {
    let fs = TestFileSystem::new().fail("mkdir");
    assert!(add_task_with_fs("New".to_string(), None, &fs).is_err());
}

#[test]
fn test_add_task_error_on_append() {
    let fs = TestFileSystem::new().fail("append");
    assert!(add_task_with_fs("New".to_string(), None, &fs).is_err());
}

#[test]
fn test_add_task_error_on_write() {
    let fs = TestFileSystem::new().fail("write");
    assert!(add_task_with_fs("New".to_string(), None, &fs).is_err());
}

#[test]
fn test_mark_task_done_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(mark_task_done_with_fs("1", &fs).is_err());
}

#[test]
fn test_mark_task_done_error_on_write() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("write");
    assert!(mark_task_done_with_fs("1", &fs).is_err());
}

#[test]
fn test_increment_pain_count_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(increment_pain_count_with_fs("1", &fs).is_err());
}

#[test]
fn test_increment_pain_count_error_on_write() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("write");
    assert!(increment_pain_count_with_fs("1", &fs).is_err());
}

#[test]
fn test_increment_pain_count_not_found() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n");
    assert!(increment_pain_count_with_fs("999", &fs).is_err());
}

// NOTE: Wrapper functions (write_tasks, get_next_id, add_task, mark_task_done, read_tasks)
// have been removed. Main.rs now calls *_with_fs() functions directly with RealFileSystem.
// This eliminates the need for problematic tests that were causing the data loss bug in task-114.

#[test]
fn test_real_filesystem_open_nonexistent_file() {
    let fs = RealFileSystem;
    let result = fs.open(Path::new("/nonexistent/file/that/does/not/exist.txt"));
    assert!(result.is_err());
}

#[test]
fn test_find_next_task_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(find_next_task_with_fs(&fs).is_err());
}

#[test]
fn test_mark_task_done_with_malformed_oldest_task_id() {
    // Test the unwrap_or(i32::MAX) fallback when parsing task IDs
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "abc|open|Malformed ID task\n2|open|Normal task\n");
    // Mark task-2 as done, which should try to compare IDs and hit the parse error fallback
    let result = mark_task_done_with_fs("2", &fs);
    assert!(result.is_ok());
}

#[test]
fn test_find_next_task_with_malformed_task_id() {
    // Test the unwrap_or(0) fallback when parsing task IDs in find_next_task_with_fs
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "abc|open|Malformed ID task\n2|open|Normal task\n");
    let result = find_next_task_with_fs(&fs);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_mark_task_done_with_duplicate_task_ids() {
    // Test edge case where multiple tasks have the same ID (malformed data)
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "5|open|Task five\n5|open|Duplicate task five\n");
    let result = mark_task_done_with_fs("5", &fs);
    assert!(result.is_ok());
}

#[test]
fn test_mark_task_done_when_no_skipped_task_found() {
    // Edge case: oldest task ID doesn't exist in the list (should never happen, but test the branch)
    // This tests the case where we exit the inner loop without finding the skipped task
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "10|open|Task ten\n");
    // Mark task-10 as done - it's the only/oldest task, so no skip happens
    let result = mark_task_done_with_fs("10", &fs);
    assert!(result.is_ok());
}

#[test]
fn test_mark_task_done_when_all_tasks_will_be_done() {
    // Edge case: marking the last open task as done (no open tasks remain after)
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|done|Already done\n2|open|Last open task\n");
    let result = mark_task_done_with_fs("2", &fs);
    assert!(result.is_ok());
}

#[test]
fn test_mark_task_done_iterates_through_multiple_tasks() {
    // Test case where we iterate through multiple tasks before finding the skipped task
    // This covers the loop path where we check multiple tasks and hit line 295 (closing brace)
    // Create multiple tasks where oldest is last in the list
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "10|open|Task ten\n5|open|Task five (oldest)\n20|open|Task twenty\n");
    // Mark task-20 as done - oldest is task-5, so we'll iterate through task-10 first (no match)
    // then find task-5 and increment its pain
    let result = mark_task_done_with_fs("20", &fs);
    assert!(result.is_ok());
}

#[test]
fn test_real_filesystem_create_in_nonexistent_directory() {
    let fs = RealFileSystem;
    // Try to create a file in a path that requires a non-existent directory
    let result = fs.create(Path::new("/nonexistent/impossible/path/file.txt"));
    assert!(result.is_err());
}

#[test]
fn test_real_filesystem_append_nonexistent_parent() {
    let fs = RealFileSystem;
    // Try to append to a file in a non-existent directory
    let result = fs.append(Path::new("/nonexistent/impossible/path/for/append/test.txt"));
    assert!(result.is_err());
}

#[test]
fn test_find_task_by_id_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(find_task_by_id_with_fs("1", &fs).is_err());
}

#[test]
fn test_find_task_by_id_not_found() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n");
    let result = find_task_by_id_with_fs("999", &fs);
    assert!(result.is_err());
}

#[test]
fn test_delete_task_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(delete_task_with_fs("1", &fs).is_err());
}

#[test]
fn test_delete_task_error_on_write() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n2|open|Another\n").fail("write");
    assert!(delete_task_with_fs("1", &fs).is_err());
}

#[test]
fn test_delete_task_not_found() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n");
    let result = delete_task_with_fs("999", &fs);
    assert!(result.is_err());
}

#[test]
fn test_update_task_error_on_read() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("open");
    assert!(update_task_with_fs("1", Some("New".to_string()), None, &fs).is_err());
}

#[test]
fn test_update_task_error_on_write() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("write");
    assert!(update_task_with_fs("1", Some("New".to_string()), None, &fs).is_err());
}

#[test]
fn test_update_task_not_found() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n");
    let result = update_task_with_fs("999", Some("New".to_string()), None, &fs);
    assert!(result.is_err());
}

#[test]
fn test_update_task_title_only() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|OldTitle\n");
    let result = update_task_with_fs("1", Some("NewTitle".to_string()), None, &fs);
    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.title, "NewTitle");
}

#[test]
fn test_update_task_description_only() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Title|OldDesc\n");
    let result = update_task_with_fs("1", None, Some(Some("NewDesc".to_string())), &fs);
    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.description, Some("NewDesc".to_string()));
}

#[test]
fn test_update_task_clear_description() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Title|Description\n");
    let result = update_task_with_fs("1", None, Some(None), &fs);
    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.description, None);
}

#[test]
fn test_update_task_both_fields() {
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|OldTitle|OldDesc\n");
    let result = update_task_with_fs("1", Some("NewTitle".to_string()), Some(Some("NewDesc".to_string())), &fs);
    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.title, "NewTitle");
    assert_eq!(task.description, Some("NewDesc".to_string()));
}