//! Memory system for persistent agent memory.

use pyo3::prelude::*;
use pyo3::types::PyList;
use std::fs;
use std::path::PathBuf;

/// Memory system for the agent.
///
/// Supports daily notes (memory/YYYY-MM-DD.md) and long-term memory (MEMORY.md).
#[pyclass]
pub struct MemoryStore {
    workspace: PathBuf,
    memory_dir: PathBuf,
    memory_file: PathBuf,
}

#[pymethods]
impl MemoryStore {
    #[new]
    pub fn new(workspace: PathBuf) -> PyResult<Self> {
        let memory_dir = workspace.join("memory");

        // Ensure directory exists
        fs::create_dir_all(&memory_dir).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create memory directory: {}",
                e
            ))
        })?;

        let memory_file = memory_dir.join("MEMORY.md");

        Ok(MemoryStore {
            workspace,
            memory_dir,
            memory_file,
        })
    }

    /// Get path to today's memory file.
    fn get_today_file(&self) -> String {
        let today = today_date();
        self.memory_dir
            .join(format!("{}.md", today))
            .to_string_lossy()
            .to_string()
    }

    /// Read today's memory notes.
    fn read_today(&self) -> String {
        let today_file = self.memory_dir.join(format!("{}.md", today_date()));
        if today_file.exists() {
            fs::read_to_string(&today_file).unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Append content to today's memory notes.
    fn append_today(&self, content: String) -> PyResult<()> {
        let today = today_date();
        let today_file = self.memory_dir.join(format!("{}.md", today));

        let final_content = if today_file.exists() {
            let existing = fs::read_to_string(&today_file).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Failed to read today's file: {}",
                    e
                ))
            })?;
            format!("{}\n{}", existing, content)
        } else {
            // Add header for new day
            format!("# {}\n\n{}", today, content)
        };

        fs::write(&today_file, final_content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to write today's file: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// Read long-term memory (MEMORY.md).
    fn read_long_term(&self) -> String {
        if self.memory_file.exists() {
            fs::read_to_string(&self.memory_file).unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// Write to long-term memory (MEMORY.md).
    fn write_long_term(&self, content: String) -> PyResult<()> {
        fs::write(&self.memory_file, content).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to write long-term memory: {}",
                e
            ))
        })?;
        Ok(())
    }

    /// Get memories from the last N days.
    #[pyo3(signature = (days=7))]
    fn get_recent_memories(&self, days: i64) -> String {
        use chrono::{Duration, Local};

        let today = Local::now().date_naive();
        let mut memories = Vec::new();

        for i in 0..days {
            let date = today - Duration::days(i);
            let date_str = date.format("%Y-%m-%d").to_string();
            let file_path = self.memory_dir.join(format!("{}.md", date_str));

            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    memories.push(content);
                }
            }
        }

        memories.join("\n\n---\n\n")
    }

    /// List all memory files sorted by date (newest first).
    fn list_memory_files(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let result = PyList::empty(py);

        if !self.memory_dir.exists() {
            return Ok(result.into());
        }

        let mut files: Vec<PathBuf> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.memory_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Match pattern YYYY-MM-DD.md
                    if name.len() == 13
                        && name.ends_with(".md")
                        && name.chars().nth(4) == Some('-')
                        && name.chars().nth(7) == Some('-')
                    {
                        files.push(path);
                    }
                }
            }
        }

        // Sort by filename (date) descending
        files.sort_by(|a, b| b.cmp(a));

        for file in files {
            result.append(file.to_string_lossy().to_string())?;
        }

        Ok(result.into())
    }

    /// Get memory context for the agent.
    pub fn get_memory_context(&self) -> String {
        let mut parts = Vec::new();

        // Long-term memory
        let long_term = self.read_long_term();
        if !long_term.is_empty() {
            parts.push(format!("## Long-term Memory\n{}", long_term));
        }

        // Today's notes
        let today = self.read_today();
        if !today.is_empty() {
            parts.push(format!("## Today's Notes\n{}", today));
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n\n")
        }
    }

    /// Get the workspace path.
    #[getter]
    fn workspace(&self) -> String {
        self.workspace.to_string_lossy().to_string()
    }

    /// Get the memory directory path.
    #[getter]
    fn memory_dir(&self) -> String {
        self.memory_dir.to_string_lossy().to_string()
    }

    /// Get the memory file path.
    #[getter]
    fn memory_file(&self) -> String {
        self.memory_file.to_string_lossy().to_string()
    }
}

/// Get today's date in YYYY-MM-DD format.
fn today_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}
