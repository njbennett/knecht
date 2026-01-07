use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fmt;

/// Escape backslashes and pipes for storage in pipe-delimited format
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('|', "\\|")
}

/// Unescape backslashes and pipes from pipe-delimited format
fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(&next_ch) = chars.peek()
                && (next_ch == '\\' || next_ch == '|') {
                    result.push(chars.next().unwrap());
                    continue;
                }
        result.push(ch);
    }
    
    result
}

/// Split a line on unescaped pipe characters
fn split_unescaped(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(&next_ch) = chars.peek()
                && (next_ch == '|' || next_ch == '\\') {
                    // Keep the escape sequence intact for later unescaping
                    current.push(ch);
                    current.push(chars.next().unwrap());
                    continue;
                }
        
        if ch == '|' {
            // Unescaped pipe - field separator
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    
    // Don't forget the last part
    parts.push(current);
    
    parts
}

#[derive(Debug)]
pub enum KnechtError {
    IoError(io::Error),
    TaskNotFound(String),
}

impl fmt::Display for KnechtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KnechtError::IoError(err) => write!(f, "I/O error: {}", err),
            KnechtError::TaskNotFound(id) => write!(f, "task-{} not found", id),
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
    pub description: Option<String>,
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
        
        let parts = split_unescaped(&line);
        if parts.len() >= 3 {
            // Support both old format (3 fields) and new format (4 fields)
            let description = if parts.len() >= 4 {
                Some(unescape(&parts[3]))
            } else {
                None
            };
            
            tasks.push(Task {
                id: parts[0].clone(),
                status: parts[1].clone(),
                title: unescape(&parts[2]),
                description,
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
        let line = if let Some(desc) = &task.description {
            format!("{}|{}|{}|{}\n", task.id, task.status, escape(&task.title), escape(desc))
        } else {
            format!("{}|{}|{}\n", task.id, task.status, escape(&task.title))
        };
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

pub fn add_task(title: String, description: Option<String>) -> Result<u32, KnechtError> {
    let next_id = get_next_id()?;
    let line = if let Some(desc) = description {
        format!("{}|open|{}|{}\n", next_id, escape(&title), escape(&desc))
    } else {
        format!("{}|open|{}\n", next_id, escape(&title))
    };
    
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