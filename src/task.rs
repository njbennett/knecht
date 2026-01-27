use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

mod serializer;
pub use serializer::CsvSerializer;

/// Trait for filesystem operations to allow dependency injection in tests
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead>>;
    fn create(&self, path: &Path) -> io::Result<Box<dyn Write>>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn append(&self, path: &Path) -> io::Result<Box<dyn Write>>;
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

/// Real filesystem implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead>> {
        let file = fs::File::open(path)?;
        Ok(Box::new(BufReader::new(file)))
    }

    fn create(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        let file = fs::File::create(path)?;
        Ok(Box::new(file))
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn append(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Box::new(file))
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}



#[derive(Debug)]
pub enum KnechtError {
    IoError(io::Error),
    CsvError(csv::Error),
    TaskNotFound(String),
    TaskAlreadyDelivered(String),
    TaskAlreadyDone(String),
}

impl fmt::Display for KnechtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KnechtError::IoError(err) => write!(f, "I/O error: {}", err),
            KnechtError::CsvError(err) => write!(f, "CSV error: {}", err),
            KnechtError::TaskNotFound(id) => write!(f, "task-{} not found", id),
            KnechtError::TaskAlreadyDelivered(id) => write!(f, "task-{} is already delivered", id),
            KnechtError::TaskAlreadyDone(id) => write!(f, "task-{} is already done", id),
        }
    }
}

impl From<io::Error> for KnechtError {
    fn from(err: io::Error) -> Self {
        KnechtError::IoError(err)
    }
}

impl From<csv::Error> for KnechtError {
    fn from(err: csv::Error) -> Self {
        KnechtError::CsvError(err)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub status: String,
    pub title: String,
    pub description: Option<String>,
    pub pain_count: Option<u32>,
    pub acceptance_criteria: Option<String>,
}

/// A single pain instance recorded in the append-only pain log
#[derive(Debug, Clone)]
pub struct PainEntry {
    pub task_id: String,
    pub timestamp: u64,
    pub source_type: PainSourceType,
    pub source_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum PainSourceType {
    Manual,
    Skip,
}

impl Task {
    pub fn is_done(&self) -> bool {
        self.status == "done"
    }

    pub fn mark_done(&mut self) {
        self.status = "done".to_string();
    }

    pub fn mark_delivered(&mut self) {
        self.status = "delivered".to_string();
    }

    pub fn mark_claimed(&mut self) {
        self.status = "claimed".to_string();
    }
}

pub fn read_tasks_with_fs(fs: &dyn FileSystem) -> Result<Vec<Task>, KnechtError> {
    let path = Path::new(".knecht/tasks");

    if !fs.exists(path) {
        return Ok(Vec::new());
    }

    // Check if it's a directory (new format) or file (old format)
    if fs.is_dir(path) {
        // New directory-based format: read each file as a single task
        let entries = fs.read_dir(path)?;
        let mut tasks = Vec::new();
        for entry in entries {
            let reader = fs.open(&entry)?;
            let mut file_tasks = CsvSerializer::read(reader)?;
            tasks.append(&mut file_tasks);
        }
        Ok(tasks)
    } else {
        // Old single-file format: read all tasks from one file
        let reader = fs.open(path)?;
        CsvSerializer::read(reader)
    }
}

/// Migrate from old single-file format to new directory-based format
pub fn migrate_to_directory_format(fs: &dyn FileSystem) -> Result<(), KnechtError> {
    let path = Path::new(".knecht/tasks");

    // Only migrate if old file format exists
    if !fs.exists(path) || fs.is_dir(path) {
        return Ok(());
    }

    // Read all tasks from old file
    let reader = fs.open(path)?;
    let tasks = CsvSerializer::read(reader)?;

    // Remove old file first
    fs.remove_file(path)?;

    // Create new directory
    fs.create_dir_all(path)?;

    // Write each task to individual file
    for task in &tasks {
        let task_path = path.join(&task.id);
        let file = fs.create(&task_path)?;
        CsvSerializer::write(std::slice::from_ref(task), file)?;
    }

    Ok(())
}

pub fn write_tasks_with_fs(tasks: &[Task], fs: &dyn FileSystem) -> Result<(), KnechtError> {
    // Migrate from old file format if needed
    migrate_to_directory_format(fs)?;

    // Ensure .knecht/tasks directory exists (new format)
    fs.create_dir_all(Path::new(".knecht/tasks"))?;

    // Write each task to its own file
    for task in tasks {
        let task_path = PathBuf::from(".knecht/tasks").join(&task.id);
        let file = fs.create(&task_path)?;
        CsvSerializer::write(std::slice::from_ref(task), file)?;
    }
    Ok(())
}

/// Writes a single task to its own file (optimized for single-task updates)
pub fn write_task_with_fs(task: &Task, fs: &dyn FileSystem) -> Result<(), KnechtError> {
    // Ensure .knecht/tasks directory exists
    fs.create_dir_all(Path::new(".knecht/tasks"))?;

    let task_path = PathBuf::from(".knecht/tasks").join(&task.id);
    let file = fs.create(&task_path)?;
    CsvSerializer::write(std::slice::from_ref(task), file)?;
    Ok(())
}

/// Generates a 6-character random alphanumeric ID using timestamp and process ID for entropy.
/// This avoids merge conflicts when parallel agents create tasks.
pub fn generate_random_id() -> String {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id();

    // Combine nanos and pid for entropy
    let mut seed = nanos as u64 ^ ((pid as u64) << 32);

    let mut id = String::with_capacity(6);
    for _ in 0..6 {
        // Simple LCG-style mixing
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = ((seed >> 32) as usize) % CHARS.len();
        id.push(CHARS[idx] as char);
    }
    id
}

pub fn add_task_with_fs(title: String, description: Option<String>, acceptance_criteria: Option<String>, fs: &dyn FileSystem) -> Result<String, KnechtError> {
    let new_id = generate_random_id();

    // Migrate from old file format if needed
    migrate_to_directory_format(fs)?;

    // Ensure .knecht/tasks directory exists
    fs.create_dir_all(Path::new(".knecht/tasks"))?;

    let task = Task {
        id: new_id.clone(),
        status: "open".to_string(),
        title,
        description,
        pain_count: None,
        acceptance_criteria,
    };

    // Create individual file for the new task
    let task_path = PathBuf::from(".knecht/tasks").join(&new_id);
    let file = fs.create(&task_path)?;
    CsvSerializer::write(std::slice::from_ref(&task), file)?;

    Ok(new_id)
}

pub fn find_task_by_id_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    let path = Path::new(".knecht/tasks");

    // Optimized: try to read single file directly if directory-based storage
    if fs.is_dir(path) {
        let task_path = path.join(task_id);
        if fs.exists(&task_path) {
            let reader = fs.open(&task_path)?;
            let tasks = CsvSerializer::read(reader)?;
            if let Some(task) = tasks.into_iter().next() {
                return Ok(task);
            }
        }
        return Err(KnechtError::TaskNotFound(task_id.to_string()));
    }

    // Fallback: read all tasks (old format)
    let tasks = read_tasks_with_fs(fs)?;
    for task in tasks {
        if task.id == task_id {
            return Ok(task);
        }
    }

    Err(KnechtError::TaskNotFound(task_id.to_string()))
}

pub fn mark_task_done_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks_with_fs(fs)?;

    // Check if task exists and is already done
    let existing_task = tasks.iter().find(|t| t.id == task_id);
    if let Some(task) = existing_task
        && task.status == "done" {
            return Err(KnechtError::TaskAlreadyDone(task_id.to_string()));
        }

    // Find the first open task (by string comparison for consistent ordering)
    let oldest_open_task_id = tasks.iter()
        .filter(|t| t.status == "open")
        .min_by(|a, b| a.id.cmp(&b.id))
        .map(|t| t.id.clone());

    // Check if the task being marked done is different from the oldest open task
    let should_increment_skip = oldest_open_task_id.as_ref().is_some_and(|oldest_id| oldest_id != task_id);
    let skipped_task_id = oldest_open_task_id.clone();

    for task in &mut tasks {
        if task.id == task_id {
            task.mark_done();
            let completed_task = task.clone();

            // If we skipped the top task, log pain to append-only pain log
            if should_increment_skip {
                if let Some(ref skipped_id) = skipped_task_id {
                    let entry = PainEntry {
                        task_id: skipped_id.clone(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        source_type: PainSourceType::Skip,
                        source_id: Some(task_id.to_string()),
                        description: format!("Skip: task-{} completed instead", task_id),
                    };
                    append_pain_entry_with_fs(&entry, fs)?;
                }
            }

            write_tasks_with_fs(&tasks, fs)?;
            return Ok(completed_task);
        }
    }

    Err(KnechtError::TaskNotFound(task_id.to_string()))
}

pub fn mark_task_delivered_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    // Optimized: read and write single task file
    let mut task = find_task_by_id_with_fs(task_id, fs)?;

    if task.status == "delivered" {
        return Err(KnechtError::TaskAlreadyDelivered(task_id.to_string()));
    }
    if task.status == "done" {
        return Err(KnechtError::TaskAlreadyDone(task_id.to_string()));
    }

    task.mark_delivered();
    write_task_with_fs(&task, fs)?;
    Ok(task)
}

pub fn mark_task_claimed_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    // Optimized: read and write single task file
    let mut task = find_task_by_id_with_fs(task_id, fs)?;
    task.mark_claimed();
    write_task_with_fs(&task, fs)?;
    Ok(task)
}

/// Returns a list of task IDs that block the given task (i.e., tasks that must be completed first)
fn get_blockers_for_task(task_id: &str, fs: &dyn FileSystem) -> Vec<String> {
    let blockers_path = Path::new(".knecht/blockers");
    
    // If blockers file doesn't exist, return empty vec
    if !fs.exists(blockers_path) {
        return Vec::new();
    }
    
    let reader = fs.open(blockers_path).expect("Failed to open blockers file");
    
    let mut blockers = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line from blockers file");

        let parts: Vec<&str> = line.split('|').collect();
        let blocked = parts[0].trim_start_matches("task-");
        let blocker = parts[1].trim_start_matches("task-");
        if blocked == task_id {
            blockers.push(blocker.to_string());
        }
    }
    blockers
}

/// Returns true if the task has any open blockers (tasks that must be completed before this one)
fn has_open_blockers(task_id: &str, tasks: &[Task], fs: &dyn FileSystem) -> bool {
    let blockers = get_blockers_for_task(task_id, fs);
    
    for blocker_id in blockers {
        if let Some(blocker_task) = tasks.iter().find(|t| t.id == blocker_id)
            && blocker_task.status == "open" {
                return true;
            }
    }
    
    false
}

/// Recursively finds the best unblocked blocker task to work on
fn find_best_blocker(task_id: &str, tasks: &[Task], pain_counts: &HashMap<String, u32>, fs: &dyn FileSystem) -> Option<Task> {
    let blockers = get_blockers_for_task(task_id, fs);

    // Get open blocker tasks
    let open_blockers: Vec<&Task> = tasks.iter()
        .filter(|t| t.status == "open" && blockers.contains(&t.id))
        .collect();

    // Find best blocker by pain count with consistent tiebreaking by ID
    let best_blocker = open_blockers.iter()
        .max_by(|a, b| {
            let pain_a = pain_counts.get(&a.id).copied().unwrap_or(0);
            let pain_b = pain_counts.get(&b.id).copied().unwrap_or(0);
            // First compare by pain count (higher is better)
            pain_a.cmp(&pain_b)
                // On tie, prefer lexicographically smaller ID (consistent ordering)
                .then_with(|| b.id.cmp(&a.id))
        })
        .map(|t| (*t).clone())
        .expect("No blocker found");

    // Check if this blocker itself has open blockers - recursively find leaf blocker
    if has_open_blockers(&best_blocker.id, tasks, fs) {
        // Recursively find the best blocker of this blocker
        return find_best_blocker(&best_blocker.id, tasks, pain_counts, fs);
    }

    Some(best_blocker)
}

/// Find the best task from a list by highest pain count, with consistent tiebreaking by ID
fn find_best_by_priority(tasks: &[&Task], pain_counts: &HashMap<String, u32>) -> Option<Task> {
    tasks.iter()
        .max_by(|a, b| {
            let pain_a = pain_counts.get(&a.id).copied().unwrap_or(0);
            let pain_b = pain_counts.get(&b.id).copied().unwrap_or(0);
            // First compare by pain count (higher is better)
            pain_a.cmp(&pain_b)
                // On tie, prefer lexicographically smaller ID (consistent ordering)
                .then_with(|| b.id.cmp(&a.id))
        })
        .map(|t| (*t).clone())
}

pub fn find_next_task_with_fs(fs: &dyn FileSystem) -> Result<Option<Task>, KnechtError> {
    let tasks = read_tasks_with_fs(fs)?;

    // Get pain counts from the pain log (efficient bulk read)
    let pain_counts = get_all_pain_counts(fs)?;

    // First, check for delivered tasks (needing verification) - they take priority
    let delivered_tasks: Vec<_> = tasks.iter()
        .filter(|t| t.status == "delivered")
        .collect();

    if !delivered_tasks.is_empty() {
        return Ok(find_best_by_priority(&delivered_tasks, &pain_counts));
    }

    // Otherwise, fall back to open tasks
    let open_tasks: Vec<_> = tasks.iter()
        .filter(|t| t.status == "open")
        .collect();

    if open_tasks.is_empty() {
        return Ok(None);
    }

    let best_task = find_best_by_priority(&open_tasks, &pain_counts);

    // If the best task has open blockers, find the best blocker to work on instead
    if let Some(ref task) = best_task
        && has_open_blockers(&task.id, &tasks, fs) {
            // find_best_blocker always returns Some (panics if no blocker found)
            let blocker = find_best_blocker(&task.id, &tasks, &pain_counts, fs).unwrap();
            return Ok(Some(blocker));
        }

    Ok(best_task)
}

/// Append a pain entry to the append-only pain log (.knecht/pain)
pub fn append_pain_entry_with_fs(entry: &PainEntry, fs: &dyn FileSystem) -> Result<(), KnechtError> {
    let pain_path = Path::new(".knecht/pain");

    let source_type_str = match entry.source_type {
        PainSourceType::Manual => "manual",
        PainSourceType::Skip => "skip",
    };
    let source_id_str = entry.source_id.as_deref().unwrap_or("");

    let mut writer = fs.append(pain_path)?;
    writeln!(writer, "{}|{}|{}|{}|{}",
        entry.task_id, entry.timestamp, source_type_str, source_id_str, entry.description)?;

    Ok(())
}

/// Read all pain entries from the pain log
pub fn read_pain_entries_with_fs(fs: &dyn FileSystem) -> Result<Vec<PainEntry>, KnechtError> {
    let pain_path = Path::new(".knecht/pain");

    if !fs.exists(pain_path) {
        return Ok(Vec::new());
    }

    let reader = fs.open(pain_path)?;
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() { continue; }

        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() >= 5 {
            entries.push(PainEntry {
                task_id: parts[0].to_string(),
                timestamp: parts[1].parse().unwrap_or(0),
                source_type: if parts[2] == "skip" { PainSourceType::Skip } else { PainSourceType::Manual },
                source_id: if parts[3].is_empty() { None } else { Some(parts[3].to_string()) },
                description: parts[4].to_string(),
            });
        }
    }

    Ok(entries)
}

/// Get pain entries for a specific task
pub fn get_pain_entries_for_task(task_id: &str, fs: &dyn FileSystem) -> Result<Vec<PainEntry>, KnechtError> {
    let entries = read_pain_entries_with_fs(fs)?;
    Ok(entries.into_iter().filter(|e| e.task_id == task_id).collect())
}

/// Get pain count for a specific task from the pain log
pub fn get_pain_count_for_task(task_id: &str, fs: &dyn FileSystem) -> Result<u32, KnechtError> {
    let entries = read_pain_entries_with_fs(fs)?;
    Ok(entries.iter().filter(|e| e.task_id == task_id).count() as u32)
}

use std::collections::HashMap;

/// Get pain counts for all tasks (more efficient for bulk operations like list)
pub fn get_all_pain_counts(fs: &dyn FileSystem) -> Result<HashMap<String, u32>, KnechtError> {
    let entries = read_pain_entries_with_fs(fs)?;
    let mut counts: HashMap<String, u32> = HashMap::new();

    for entry in entries {
        *counts.entry(entry.task_id).or_insert(0) += 1;
    }

    Ok(counts)
}

pub fn increment_pain_count_with_fs(task_id: &str, pain_description: Option<&str>, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    // Verify task exists
    let task = find_task_by_id_with_fs(task_id, fs)?;

    // Append to pain log (append-only for merge conflict reduction)
    let entry = PainEntry {
        task_id: task_id.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        source_type: PainSourceType::Manual,
        source_id: None,
        description: pain_description.unwrap_or("").to_string(),
    };
    append_pain_entry_with_fs(&entry, fs)?;

    Ok(task)
}

pub fn delete_task_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    // Read the task first to return its data
    let task = find_task_by_id_with_fs(task_id, fs)?;

    // Delete the task file
    let task_path = PathBuf::from(".knecht/tasks").join(task_id);
    fs.remove_file(&task_path)?;

    Ok(task)
}

pub fn update_task_with_fs(
    task_id: &str,
    new_title: Option<String>,
    new_description: Option<Option<String>>,
    new_acceptance_criteria: Option<Option<String>>,
    fs: &dyn FileSystem
) -> Result<Task, KnechtError> {
    // Optimized: read and write single task file
    let mut task = find_task_by_id_with_fs(task_id, fs)?;

    // Update title if provided
    if let Some(title) = new_title {
        task.title = title;
    }

    // Update description if provided
    // None = no change, Some(None) = clear description, Some(Some(desc)) = set description
    if let Some(desc_opt) = new_description {
        task.description = desc_opt;
    }

    // Update acceptance_criteria if provided
    // None = no change, Some(None) = clear criteria, Some(Some(criteria)) = set criteria
    if let Some(criteria_opt) = new_acceptance_criteria {
        task.acceptance_criteria = criteria_opt;
    }

    write_task_with_fs(&task, fs)?;
    Ok(task)
}
