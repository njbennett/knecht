use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fmt;

#[derive(Debug)]
pub enum KnechtError {
    IoError(io::Error),
    TaskNotFound(String),
    InvalidCharacter(String),
}

impl fmt::Display for KnechtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KnechtError::IoError(err) => write!(f, "I/O error: {}", err),
            KnechtError::TaskNotFound(id) => write!(f, "task-{} not found", id),
            KnechtError::InvalidCharacter(msg) => write!(f, "Invalid character: {}", msg),
        }
    }
}

impl From<io::Error> for KnechtError {
    fn from(err: io::Error) -> Self {
        KnechtError::IoError(err)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub status: String,
    pub title: String,
}

impl Task {
    pub fn is_done(&self) -> bool {
        self.status == "done"
    }

    pub fn mark_done(&mut self) {
        self.status = "done".to_string();
    }
}

pub fn read_tasks() -> Result<Vec<Task>, KnechtError> {
    let path = Path::new(".knecht/tasks");
    
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            // Use unchecked constructor since we're reading existing data
            tasks.push(Task {
                id: parts[0].to_string(),
                status: parts[1].to_string(),
                title: parts[2].to_string(),
            });
        }
        // Skip malformed lines silently
    }
    
    Ok(tasks)
}

pub fn write_tasks(tasks: &[Task]) -> Result<(), KnechtError> {
    // Ensure .knecht directory exists
    fs::create_dir_all(".knecht")?;
    
    let mut file = fs::File::create(".knecht/tasks")?;
    
    for task in tasks {
        let line = format!("{}|{}|{}\n", task.id, task.status, task.title);
        file.write_all(line.as_bytes())?;
    }
    
    Ok(())
}

pub fn get_next_id() -> Result<u32, KnechtError> {
    let tasks = read_tasks()?;
    
    let max_id = tasks
        .iter()
        .filter_map(|t| t.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    
    Ok(max_id + 1)
}

pub fn add_task(title: String) -> Result<u32, KnechtError> {
    // Validate title before proceeding
    if title.contains('|') {
        return Err(KnechtError::InvalidCharacter(
            "Task title cannot contain pipe character '|'".to_string()
        ));
    }
    
    let next_id = get_next_id()?;
    let line = format!("{}|open|{}\n", next_id, title);
    
    // Ensure .knecht directory exists
    fs::create_dir_all(".knecht")?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".knecht/tasks")?;
    
    file.write_all(line.as_bytes())?;
    
    Ok(next_id)
}

pub fn mark_task_done(task_id: &str) -> Result<Task, KnechtError> {
    let mut tasks = read_tasks()?;
    
    for task in &mut tasks {
        if task.id == task_id {
            task.mark_done();
            let completed_task = task.clone();
            write_tasks(&tasks)?;
            return Ok(completed_task);
        }
    }
    
    Err(KnechtError::TaskNotFound(task_id.to_string()))
}