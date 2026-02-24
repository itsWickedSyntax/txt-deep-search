use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check if a file appears to be binary by reading its first 8KB and looking for null bytes.
fn is_binary(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 8192];
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    buf[..n].contains(&0)
}

/// Simple glob matcher supporting `*` and `?` wildcards.
fn glob_matches(pattern: &str, name: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let nam: Vec<char> = name.chars().collect();
    glob_match_inner(&pat, &nam)
}

fn glob_match_inner(pat: &[char], nam: &[char]) -> bool {
    match (pat.first(), nam.first()) {
        (None, None) => true,
        (Some('*'), _) => {
            // Try skipping the * (match zero chars) or consuming one char from name
            glob_match_inner(&pat[1..], nam)
                || (!nam.is_empty() && glob_match_inner(pat, &nam[1..]))
        }
        (Some('?'), Some(_)) => glob_match_inner(&pat[1..], &nam[1..]),
        (Some(a), Some(b)) if a == b => glob_match_inner(&pat[1..], &nam[1..]),
        _ => false,
    }
}

/// Scan a directory recursively and return all .txt file paths that pass filters.
/// Returns (files, warnings) where warnings are error messages for inaccessible entries.
pub fn scan_directory(
    dir: &Path,
    file_filter: Option<&str>,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for entry in WalkDir::new(dir).follow_links(true) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("Warning: {}", e));
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Must have .txt extension
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("txt") => {}
            _ => continue,
        }

        // Apply file name glob filter if provided
        if let Some(pattern) = file_filter {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !glob_matches(pattern, name) {
                    continue;
                }
            } else {
                continue;
            }
        }

        // Skip binary files
        if is_binary(path) {
            continue;
        }

        files.push(path.to_path_buf());
    }

    (files, warnings)
}
