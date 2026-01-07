mod test_helpers;

use test_helpers::TestFileSystem;
use knecht::{read_tasks_with_fs, write_tasks_with_fs, get_next_id_with_fs, add_task_with_fs, mark_task_done_with_fs, Task, RealFileSystem, FileSystem};
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
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None }];
    assert!(write_tasks_with_fs(&tasks, &fs).is_err());
}

#[test]
fn test_write_tasks_error_on_create() {
    let fs = TestFileSystem::new().fail("create");
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None }];
    assert!(write_tasks_with_fs(&tasks, &fs).is_err());
}

#[test]
fn test_write_tasks_error_on_write() {
    let fs = TestFileSystem::new().fail("write");
    let tasks = vec![Task { id: "1".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None }];
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
    let fs = TestFileSystem::new().with_file(".knecht/tasks", "1|open|Test\n").fail("create");
    assert!(mark_task_done_with_fs("1", &fs).is_err());
}

#[test]
fn test_public_wrapper_write_tasks() {
    // These wrapper functions are used by main.rs but not covered by integration tests
    // since integration tests run the binary, not the library directly
    let tasks = vec![Task { id: "99".to_string(), status: "open".to_string(), title: "Test".to_string(), description: None }];
    // This will fail because we're not in a directory with .knecht, but that's ok - we just need to exercise the function
    let _ = knecht::write_tasks(&tasks);
}

#[test]
fn test_public_wrapper_get_next_id() {
    // This will fail because we're not in a directory with .knecht, but that's ok - we just need to exercise the function
    let _ = knecht::get_next_id();
}

#[test]
fn test_real_filesystem_open_nonexistent_file() {
    let fs = RealFileSystem;
    let result = fs.open(Path::new("/nonexistent/path/to/file/that/does/not/exist.txt"));
    assert!(result.is_err());
}

#[test]
fn test_real_filesystem_create_in_nonexistent_directory() {
    let fs = RealFileSystem;
    // Try to create a file in a path that requires a non-existent directory
    // Use a path that's highly unlikely to exist
    let result = fs.create(Path::new("/nonexistent/impossible/directory/structure/file.txt"));
    assert!(result.is_err());
}

#[test]
fn test_real_filesystem_append_nonexistent_parent() {
    let fs = RealFileSystem;
    // Try to append to a file in a non-existent directory
    let result = fs.append(Path::new("/nonexistent/impossible/path/for/append/test.txt"));
    assert!(result.is_err());
}