use knecht::{FileSystem, Task, KnechtError};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct TestFileSystem {
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    fail_mode: Option<&'static str>,
}

impl TestFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            fail_mode: None,
        }
    }

    pub fn with_file(self, path: &str, content: &str) -> Self {
        self.files.lock().unwrap().insert(PathBuf::from(path), content.as_bytes().to_vec());
        self
    }

    pub fn fail(mut self, mode: &'static str) -> Self {
        self.fail_mode = Some(mode);
        self
    }
}

struct TestReader {
    content: Vec<u8>,
    position: usize,
    fail: bool,
}

impl BufRead for TestReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&self.content[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position += amt;
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::Other, "test error"));
        }
        if self.position >= self.content.len() {
            return Ok(0);
        }
        let start = self.position;
        while self.position < self.content.len() && self.content[self.position] != b'\n' {
            self.position += 1;
        }
        if self.position < self.content.len() {
            self.position += 1;
        }
        buf.push_str(&String::from_utf8_lossy(&self.content[start..self.position]));
        Ok(self.position - start)
    }
}

impl io::Read for TestReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::Other, "test error"));
        }
        let len = std::cmp::min(buf.len(), self.content.len() - self.position);
        buf[..len].copy_from_slice(&self.content[self.position..self.position + len]);
        self.position += len;
        Ok(len)
    }
}

struct TestWriter {
    content: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    path: PathBuf,
    fail: bool,
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::Other, "test error"));
        }
        self.content.lock().unwrap().entry(self.path.clone()).or_insert_with(Vec::new).extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl FileSystem for TestFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    fn open(&self, path: &Path) -> io::Result<Box<dyn BufRead>> {
        if self.fail_mode == Some("open") {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "test error"));
        }
        let content = self.files.lock().unwrap().get(path).cloned().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))?;
        Ok(Box::new(TestReader { content, position: 0, fail: self.fail_mode == Some("read") }))
    }

    fn create(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        if self.fail_mode == Some("create") {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "test error"));
        }
        Ok(Box::new(TestWriter { content: Arc::clone(&self.files), path: path.to_path_buf(), fail: self.fail_mode == Some("write") }))
    }

    fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
        if self.fail_mode == Some("mkdir") {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "test error"));
        }
        Ok(())
    }

    fn append(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        if self.fail_mode == Some("append") {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "test error"));
        }
        Ok(Box::new(TestWriter { content: Arc::clone(&self.files), path: path.to_path_buf(), fail: self.fail_mode == Some("write") }))
    }
}