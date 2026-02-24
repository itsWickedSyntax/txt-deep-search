use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use rayon::prelude::*;
use regex::Regex;

/// A single search match.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}

/// Aggregated results from a search run.
pub struct SearchResults {
    pub matches: Vec<SearchMatch>,
    pub files_scanned: usize,
    pub warnings: Vec<String>,
}

/// Threshold in bytes above which we use mmap instead of buffered reader.
const MMAP_THRESHOLD: u64 = 10 * 1024 * 1024; // 10 MB

/// Search a single file using a buffered reader (streaming, low memory).
fn search_file_buffered(path: &Path, re: &Regex) -> Result<Vec<SearchMatch>, String> {
    let file =
        File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    let reader = BufReader::with_capacity(64 * 1024, file);
    let mut matches = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue, // skip malformed encoding lines
        };
        if re.is_match(&line) {
            matches.push(SearchMatch {
                file_path: path.to_path_buf(),
                line_number: idx + 1,
                line_content: line,
            });
        }
    }

    Ok(matches)
}

/// Search a single file using memory-mapped I/O (efficient for large files).
fn search_file_mmap(path: &Path, re: &Regex) -> Result<Vec<SearchMatch>, String> {
    let file =
        File::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    let mmap =
        unsafe { Mmap::map(&file) }.map_err(|e| format!("Cannot mmap {}: {}", path.display(), e))?;

    let content = String::from_utf8_lossy(&mmap);
    let mut matches = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if re.is_match(line) {
            matches.push(SearchMatch {
                file_path: path.to_path_buf(),
                line_number: idx + 1,
                line_content: line.to_string(),
            });
        }
    }

    Ok(matches)
}

/// Search a single file, choosing strategy based on file size.
fn search_file(path: &Path, re: &Regex) -> Result<Vec<SearchMatch>, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Cannot stat {}: {}", path.display(), e))?;

    if metadata.len() == 0 {
        return Ok(Vec::new());
    }

    if metadata.len() >= MMAP_THRESHOLD {
        search_file_mmap(path, re)
    } else {
        search_file_buffered(path, re)
    }
}

/// Run a parallel search across all provided files.
pub fn parallel_search(
    files: &[PathBuf],
    pattern: &str,
    ignore_case: bool,
) -> SearchResults {
    let re = if ignore_case {
        Regex::new(&format!("(?i){}", pattern))
    } else {
        Regex::new(pattern)
    };

    let re = match re {
        Ok(r) => r,
        Err(e) => {
            return SearchResults {
                matches: Vec::new(),
                files_scanned: 0,
                warnings: vec![format!("Invalid regex pattern: {}", e)],
            };
        }
    };

    let total = files.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files ({eta} remaining)"
        )
        .unwrap()
        .progress_chars("█▓░"),
    );

    let warnings: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let all_matches: Vec<SearchMatch> = files
        .par_iter()
        .flat_map(|path| {
            let result = search_file(path, &re);
            pb.inc(1);
            match result {
                Ok(m) => m,
                Err(e) => {
                    warnings.lock().unwrap().push(e);
                    Vec::new()
                }
            }
        })
        .collect();

    pb.finish_and_clear();

    let warns = match Arc::try_unwrap(warnings) {
        Ok(mutex) => mutex.into_inner().unwrap_or_default(),
        Err(arc) => arc.lock().unwrap().clone(),
    };

    SearchResults {
        matches: all_matches,
        files_scanned: files.len(),
        warnings: warns,
    }
}
