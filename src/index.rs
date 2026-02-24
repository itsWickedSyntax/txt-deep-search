use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

const INDEX_DIR: &str = ".txt_index";
const INDEX_FILE: &str = "index.bin";

/// An entry in the inverted index: maps a word to a list of (file_path, line_number) pairs.
#[derive(Serialize, Deserialize, Debug)]
pub struct InvertedIndex {
    /// Maps lowercase word -> Vec<(file_index, line_number)>
    pub postings: HashMap<String, Vec<(u32, u32)>>,
    /// File index -> file path
    pub files: Vec<String>,
}

impl InvertedIndex {
    fn new() -> Self {
        Self {
            postings: HashMap::new(),
            files: Vec::new(),
        }
    }
}

/// Get the index directory path for a given base directory.
fn index_dir_for(base: &Path) -> PathBuf {
    base.join(INDEX_DIR)
}

/// Get the index file path for a given base directory.
fn index_path_for(base: &Path) -> PathBuf {
    index_dir_for(base).join(INDEX_FILE)
}

/// Build the inverted index for the given list of .txt files.
pub fn build_index(base_dir: &Path, files: &[PathBuf]) -> Result<(), String> {
    let mut index = InvertedIndex::new();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} Indexing [{bar:40.cyan/blue}] {pos}/{len} files"
        )
        .unwrap()
        .progress_chars("█▓░"),
    );

    for (file_idx, path) in files.iter().enumerate() {
        let file_idx = file_idx as u32;
        index.files.push(path.display().to_string());

        let file = File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
        let reader = BufReader::with_capacity(64 * 1024, file);

        for (line_idx, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let line_num = (line_idx + 1) as u32;

            // Tokenize: split on non-alphanumeric, lowercase
            for word in line
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
            {
                let key = word.to_lowercase();
                index
                    .postings
                    .entry(key)
                    .or_default()
                    .push((file_idx, line_num));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Serialize and write
    let idx_dir = index_dir_for(base_dir);
    fs::create_dir_all(&idx_dir)
        .map_err(|e| format!("Cannot create index directory: {}", e))?;

    let encoded = bincode::serialize(&index)
        .map_err(|e| format!("Serialization error: {}", e))?;

    let mut f = File::create(index_path_for(base_dir))
        .map_err(|e| format!("Cannot create index file: {}", e))?;
    f.write_all(&encoded)
        .map_err(|e| format!("Cannot write index: {}", e))?;

    let size_mb = encoded.len() as f64 / (1024.0 * 1024.0);
    eprintln!(
        "Index built: {} files indexed, {:.2} MB index size",
        files.len(),
        size_mb
    );

    Ok(())
}

/// Load the inverted index from disk.
pub fn load_index(base_dir: &Path) -> Result<InvertedIndex, String> {
    let path = index_path_for(base_dir);
    let data = fs::read(&path)
        .map_err(|e| format!("Cannot read index at {}: {}", path.display(), e))?;
    let index: InvertedIndex = bincode::deserialize(&data)
        .map_err(|e| format!("Index deserialization error: {}", e))?;
    Ok(index)
}

/// Query the inverted index. Returns (file_path, line_number) pairs.
pub fn query_index(index: &InvertedIndex, term: &str) -> Vec<(String, u32)> {
    let key = term.to_lowercase();
    match index.postings.get(&key) {
        Some(entries) => entries
            .iter()
            .map(|(file_idx, line_num)| {
                let path = index.files.get(*file_idx as usize).cloned().unwrap_or_default();
                (path, *line_num)
            })
            .collect(),
        None => Vec::new(),
    }
}

/// Clear the index directory for a given base directory.
pub fn clear_index(base_dir: &Path) -> Result<(), String> {
    let idx_dir = index_dir_for(base_dir);
    if idx_dir.exists() {
        fs::remove_dir_all(&idx_dir)
            .map_err(|e| format!("Cannot remove index directory: {}", e))?;
        eprintln!("Index cleared for {}", base_dir.display());
    } else {
        eprintln!("No index found for {}", base_dir.display());
    }
    Ok(())
}

/// Check if an index exists for a given directory.
pub fn index_exists(base_dir: &Path) -> bool {
    index_path_for(base_dir).exists()
}
