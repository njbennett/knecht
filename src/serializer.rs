use std::io::{BufRead, Write};
use csv::{ReaderBuilder, WriterBuilder};

use crate::{Task, KnechtError};

/// Handles CSV serialization/deserialization of tasks
pub struct CsvSerializer;

impl CsvSerializer {
    /// Read tasks from a CSV reader
    pub fn read(reader: impl BufRead) -> Result<Vec<Task>, KnechtError> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);

        let mut tasks = Vec::new();

        for result in csv_reader.records() {
            let record = result?;

            if record.len() >= 3 {
                // Support formats: id,status,title or id,status,title,description or id,status,title,description,pain_count
                let description = if record.len() >= 4 && !record[3].is_empty() {
                    Some(record[3].to_string())
                } else {
                    None
                };

                let pain_count = if record.len() >= 5 && !record[4].is_empty() {
                    record[4].parse::<u32>().ok()
                } else {
                    None
                };

                tasks.push(Task {
                    id: record[0].to_string(),
                    status: record[1].to_string(),
                    title: record[2].to_string(),
                    description,
                    pain_count,
                });
            }
            // Skip malformed lines silently
        }

        Ok(tasks)
    }

    /// Write tasks to a CSV writer
    pub fn write(tasks: &[Task], writer: impl Write) -> Result<(), KnechtError> {
        let mut csv_writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);

        for task in tasks {
            // Always write 5 fields: id, status, title, description, pain_count
            csv_writer.write_record([
                &task.id,
                &task.status,
                &task.title,
                task.description.as_deref().unwrap_or(""),
                task.pain_count.as_ref().map(|p: &u32| p.to_string()).unwrap_or_default().as_str(),
            ])?;
        }

        csv_writer.flush()?;

        Ok(())
    }

    /// Append a single task to a CSV writer
    pub fn append_task(task: &Task, writer: impl Write) -> Result<(), KnechtError> {
        let mut csv_writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(writer);

        csv_writer.write_record([
            &task.id,
            &task.status,
            &task.title,
            task.description.as_deref().unwrap_or(""),
            task.pain_count.as_ref().map(|p: &u32| p.to_string()).unwrap_or_default().as_str(),
        ])?;

        csv_writer.flush()?;

        Ok(())
    }
}
