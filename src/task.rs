use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fmt;

mod serializer;
pub use serializer::CsvSerializer;

/// Trait for filesystem operations to allow dependency injection in tests
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead>>;
    fn create(&self, path: &Path) -> io::Result<Box<dyn Write>>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn append(&self, path: &Path) -> io::Result<Box<dyn Write>>;
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
}

pub fn read_tasks_with_fs(fs: &dyn FileSystem) -> Result<Vec<Task>, KnechtError> {
    let path = Path::new(".knecht/tasks");

    if !fs.exists(path) {
        return Ok(Vec::new());
    }

    let reader = fs.open(path)?;
    CsvSerializer::read(reader)
}

pub fn write_tasks_with_fs(tasks: &[Task], fs: &dyn FileSystem) -> Result<(), KnechtError> {
    // Ensure .knecht directory exists
    fs.create_dir_all(Path::new(".knecht"))?;

    let file = fs.create(Path::new(".knecht/tasks"))?;
    CsvSerializer::write(tasks, file)
}

pub fn get_next_id_with_fs(fs: &dyn FileSystem) -> Result<u32, KnechtError> {
    let tasks = read_tasks_with_fs(fs)?;
    
    let max_id = tasks
        .iter()
        .filter_map(|t| t.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    
    Ok(max_id + 1)
}

pub fn add_task_with_fs(title: String, description: Option<String>, fs: &dyn FileSystem) -> Result<u32, KnechtError> {
    let next_id = get_next_id_with_fs(fs)?;

    // Ensure .knecht directory exists
    fs.create_dir_all(Path::new(".knecht"))?;

    let task = Task {
        id: next_id.to_string(),
        status: "open".to_string(),
        title,
        description,
        pain_count: None,
    };

    let file = fs.append(Path::new(".knecht/tasks"))?;
    CsvSerializer::append_task(&task, file)?;

    Ok(next_id)
}

pub fn find_task_by_id_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
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
    
    // Find the oldest open task (lowest ID among open tasks)
    let oldest_open_task_id = tasks.iter()
        .filter(|t| t.status == "open")
        .min_by_key(|t| t.id.parse::<i32>().unwrap_or(i32::MAX))
        .map(|t| t.id.clone());
    
    // Check if the task being marked done is different from the oldest open task
    let should_increment_skip = oldest_open_task_id.as_ref().is_some_and(|oldest_id| oldest_id != task_id);
    let skipped_task_id = oldest_open_task_id.clone();
    
    for task in &mut tasks {
        if task.id == task_id {
            task.mark_done();
            let completed_task = task.clone();
            
            // If we skipped the top task, increment its pain
            if should_increment_skip
                && let Some(ref skipped_id) = skipped_task_id {
                    for t in &mut tasks {
                        if &t.id == skipped_id {
                            t.pain_count = Some(t.pain_count.unwrap_or(0) + 1);
                            
                            // Add skip note to description
                            let skip_note = format!("Skip: task-{} completed instead", task_id);
                            if let Some(ref desc) = t.description {
                                t.description = Some(format!("{}. {}", desc, skip_note));
                            } else {
                                t.description = Some(skip_note);
                            }
                            break;
                        }
                    }
                }
            
            write_tasks_with_fs(&tasks, fs)?;
            return Ok(completed_task);
        }
    }
    
    Err(KnechtError::TaskNotFound(task_id.to_string()))
}

pub fn mark_task_delivered_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks_with_fs(fs)?;

    for task in &mut tasks {
        if task.id == task_id {
            if task.status == "delivered" {
                return Err(KnechtError::TaskAlreadyDelivered(task_id.to_string()));
            }
            if task.status == "done" {
                return Err(KnechtError::TaskAlreadyDone(task_id.to_string()));
            }
            task.mark_delivered();
            let delivered_task = task.clone();
            write_tasks_with_fs(&tasks, fs)?;
            return Ok(delivered_task);
        }
    }

    Err(KnechtError::TaskNotFound(task_id.to_string()))
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
fn find_best_blocker(task_id: &str, tasks: &[Task], fs: &dyn FileSystem) -> Option<Task> {
    let blockers = get_blockers_for_task(task_id, fs);
    
    // Get open blocker tasks
    let open_blockers: Vec<&Task> = tasks.iter()
        .filter(|t| t.status == "open" && blockers.contains(&t.id))
        .collect();
    

    
    // Find best blocker by pain count and age
    let best_blocker = open_blockers.iter()
        .max_by_key(|t| {
            let pain = t.pain_count.unwrap_or(0);
            let id_num: i32 = t.id.parse().unwrap_or(0);
            (pain, -id_num)
        })
        .map(|t| (*t).clone())
        .expect("No blocker found");
    
    // Check if this blocker itself has open blockers - recursively find leaf blocker
    if has_open_blockers(&best_blocker.id, tasks, fs) {
        // Recursively find the best blocker of this blocker
        return find_best_blocker(&best_blocker.id, tasks, fs);
    }
    
    Some(best_blocker)
}

pub fn find_next_task_with_fs(fs: &dyn FileSystem) -> Result<Option<Task>, KnechtError> {
    let tasks = read_tasks_with_fs(fs)?;
    
    // Filter to open tasks only
    let open_tasks: Vec<_> = tasks.iter()
        .filter(|t| t.status == "open")
        .collect();
    
    if open_tasks.is_empty() {
        return Ok(None);
    }
    
    // Find task with highest pain count, preferring older tasks on tie
    let best_task = open_tasks.iter()
        .max_by_key(|t| {
            let pain = t.pain_count.unwrap_or(0);
            let id_num: i32 = t.id.parse().unwrap_or(0);
            (pain, -id_num)
        })
        .map(|t| (*t).clone());
    
    // If the best task has open blockers, find the best blocker to work on instead
    if let Some(ref task) = best_task
        && has_open_blockers(&task.id, &tasks, fs) {
            // find_best_blocker always returns Some (panics if no blocker found)
            let blocker = find_best_blocker(&task.id, &tasks, fs).unwrap();
            return Ok(Some(blocker));
        }
    
    Ok(best_task)
}

pub fn increment_pain_count_with_fs(task_id: &str, pain_description: Option<&str>, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks_with_fs(fs)?;

    for task in &mut tasks {
        if task.id == task_id {
            // Increment pain_count field
            task.pain_count = Some(task.pain_count.unwrap_or(0) + 1);

            // Append pain description if provided
            if let Some(desc) = pain_description {
                let pain_note = format!("Pain: {}", desc);
                task.description = Some(match &task.description {
                    Some(existing) => format!("{}\n{}", existing, pain_note),
                    None => pain_note,
                });
            }

            let updated_task = task.clone();
            write_tasks_with_fs(&tasks, fs)?;
            return Ok(updated_task);
        }
    }

    Err(KnechtError::TaskNotFound(task_id.to_string()))
}

pub fn delete_task_with_fs(task_id: &str, fs: &dyn FileSystem) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks_with_fs(fs)?;
    
    // Find the task to delete
    let mut deleted_task = None;
    tasks.retain(|task| {
        if task.id == task_id {
            deleted_task = Some(task.clone());
            false // Remove this task
        } else {
            true // Keep this task
        }
    });
    
    match deleted_task {
        Some(task) => {
            write_tasks_with_fs(&tasks, fs)?;
            Ok(task)
        }
        None => Err(KnechtError::TaskNotFound(task_id.to_string()))
    }
}

pub fn update_task_with_fs(
    task_id: &str,
    new_title: Option<String>,
    new_description: Option<Option<String>>,
    fs: &dyn FileSystem
) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks_with_fs(fs)?;
    
    for task in &mut tasks {
        if task.id == task_id {
            // Update title if provided
            if let Some(title) = new_title {
                task.title = title;
            }
            
            // Update description if provided
            // None = no change, Some(None) = clear description, Some(Some(desc)) = set description
            if let Some(desc_opt) = new_description {
                task.description = desc_opt;
            }
            
            let updated_task = task.clone();
            write_tasks_with_fs(&tasks, fs)?;
            return Ok(updated_task);
        }
    }
    
    Err(KnechtError::TaskNotFound(task_id.to_string()))
}
