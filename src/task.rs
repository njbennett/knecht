use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub status: String,
    pub title: String,
}

impl Task {
    pub fn new(id: String, status: String, title: String) -> Self {
        Task { id, status, title }
    }

    pub fn is_done(&self) -> bool {
        self.status == "done"
    }

    pub fn mark_done(&mut self) {
        self.status = "done".to_string();
    }
}

pub fn read_tasks() -> Vec<Task> {
    let path = Path::new(".knecht/tasks");
    
    if !path.exists() {
        return Vec::new();
    }
    
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    
    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    
    for line in reader.lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            tasks.push(Task::new(
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
    }
    
    tasks
}

pub fn write_tasks(tasks: &[Task]) {
    let mut file = fs::File::create(".knecht/tasks")
        .expect("Failed to open tasks file for writing");
    
    for task in tasks {
        let line = format!("{}|{}|{}\n", task.id, task.status, task.title);
        file.write_all(line.as_bytes())
            .expect("Failed to write task");
    }
}

pub fn get_next_id() -> u32 {
    let tasks = read_tasks();
    
    let max_id = tasks
        .iter()
        .filter_map(|t| t.id.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    
    max_id + 1
}

pub fn add_task(title: String) -> u32 {
    let next_id = get_next_id();
    let line = format!("{}|open|{}\n", next_id, title);
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".knecht/tasks")
        .expect("Failed to open tasks file");
    
    file.write_all(line.as_bytes())
        .expect("Failed to write task");
    
    next_id
}

pub fn mark_task_done(task_id: &str) -> Result<Task, String> {
    let mut tasks = read_tasks();
    
    for task in &mut tasks {
        if task.id == task_id {
            task.mark_done();
            let completed_task = task.clone();
            write_tasks(&tasks);
            return Ok(completed_task);
        }
    }
    
    Err(format!("task-{} not found", task_id))
}