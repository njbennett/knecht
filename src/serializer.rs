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

                let acceptance_criteria = if record.len() >= 6 && !record[5].is_empty() {
                    Some(record[5].to_string())
                } else {
                    None
                };

                tasks.push(Task {
                    id: record[0].to_string(),
                    status: record[1].to_string(),
                    title: record[2].to_string(),
                    description,
                    pain_count,
                    acceptance_criteria,
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
            // Always write 6 fields: id, status, title, description, pain_count, acceptance_criteria
            let pain_str = task.pain_count.map(|p| p.to_string()).unwrap_or_default();
            csv_writer.write_record([
                &task.id,
                &task.status,
                &task.title,
                task.description.as_deref().unwrap_or(""),
                pain_str.as_str(),
                task.acceptance_criteria.as_deref().unwrap_or(""),
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

        let pain_str = task.pain_count.map(|p| p.to_string()).unwrap_or_default();
        csv_writer.write_record([
            &task.id,
            &task.status,
            &task.title,
            task.description.as_deref().unwrap_or(""),
            pain_str.as_str(),
            task.acceptance_criteria.as_deref().unwrap_or(""),
        ])?;

        csv_writer.flush()?;

        Ok(())
    }
}
