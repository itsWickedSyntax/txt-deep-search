use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Duration;

use colored::Colorize;
use regex::Regex;

use crate::search::SearchMatch;

/// Print all search results to stdout.
pub fn print_results(
    matches: &[SearchMatch],
    pattern: &str,
    ignore_case: bool,
    files_scanned: usize,
    elapsed: Duration,
    warnings: &[String],
    use_color: bool,
) {
    // Print warnings first
    for w in warnings {
        if use_color {
            eprintln!("{}", w.yellow());
        } else {
            eprintln!("{}", w);
        }
    }

    let re_str = if ignore_case {
        format!("(?i){}", pattern)
    } else {
        pattern.to_string()
    };
    let re = Regex::new(&re_str).unwrap_or_else(|_| Regex::new("$^").unwrap());

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for m in matches {
        let path_str = m.file_path.display().to_string();
        if use_color {
            let highlighted = re.replace_all(&m.line_content, |caps: &regex::Captures| {
                caps[0].red().bold().to_string()
            });
            let _ = writeln!(
                out,
                "{}{}{}  {}",
                path_str.green(),
                ":".dimmed(),
                m.line_number.to_string().yellow(),
                highlighted
            );
        } else {
            let _ = writeln!(out, "{}:{}  {}", path_str, m.line_number, m.line_content);
        }
    }

    let _ = out.flush();

    // Summary
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    eprintln!();
    if use_color {
        eprintln!(
            "{} {} matches across {} files scanned in {:.1}ms",
            "Results:".bold(),
            matches.len(),
            files_scanned,
            elapsed_ms
        );
    } else {
        eprintln!(
            "Results: {} matches across {} files scanned in {:.1}ms",
            matches.len(),
            files_scanned,
            elapsed_ms
        );
    }
}

/// Export results to a CSV file.
pub fn export_csv(matches: &[SearchMatch], path: &Path) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path)
        .map_err(|e| format!("Cannot create CSV file {}: {}", path.display(), e))?;

    wtr.write_record(["file_path", "line_number", "matched_line"])
        .map_err(|e| format!("CSV write error: {}", e))?;

    for m in matches {
        wtr.write_record([
            m.file_path.display().to_string(),
            m.line_number.to_string(),
            m.line_content.clone(),
        ])
        .map_err(|e| format!("CSV write error: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("CSV flush error: {}", e))?;

    eprintln!("Results exported to {}", path.display());
    Ok(())
}
