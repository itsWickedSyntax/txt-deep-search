use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::Parser;
use colored::Colorize;

use txt_deep_search::cli::Cli;
use txt_deep_search::{index, output, scanner, search};

fn main() {
    let args = Cli::parse();

    // --- Handle --clear-index ---
    if let Some(ref dir) = args.clear_index {
        if let Err(e) = index::clear_index(dir) {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
        return;
    }

    // --- Handle --reindex ---
    if let Some(ref dir) = args.reindex {
        eprintln!("Scanning {} ...", dir.display());
        let (files, warnings) = scanner::scan_directory(dir, args.file_filter.as_deref());
        for w in &warnings {
            eprintln!("{}", w.yellow());
        }
        eprintln!("Found {} .txt files", files.len());
        if let Err(e) = index::build_index(dir, &files) {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
        return;
    }

    // --- Search mode: need a directory and a query/regex ---
    let dir = match args.directory {
        Some(ref d) => d.clone(),
        None => {
            eprintln!("{} No directory specified. Run with --help for usage.", "Error:".red().bold());
            process::exit(1);
        }
    };

    if !dir.is_dir() {
        eprintln!("{} '{}' is not a directory", "Error:".red().bold(), dir.display());
        process::exit(1);
    }

    let pattern = match args.build_pattern() {
        Some(p) => p,
        None => {
            eprintln!("{} No search query specified. Use --query or --regex.", "Error:".red().bold());
            process::exit(1);
        }
    };

    // Configure thread pool
    if let Some(n) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }

    let use_color = atty::is(atty::Stream::Stdout);

    // --- Indexed search mode ---
    if args.index {
        // Build index if it doesn't exist
        if !index::index_exists(&dir) {
            eprintln!("No index found. Building index...");
            let (files, warnings) = scanner::scan_directory(&dir, args.file_filter.as_deref());
            for w in &warnings {
                eprintln!("{}", w.yellow());
            }
            if let Err(e) = index::build_index(&dir, &files) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        }

        let start = Instant::now();
        let idx = match index::load_index(&dir) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        };

        // For indexed search, use the raw query term (works best with single words)
        let term = args.query.as_deref().unwrap_or(
            args.regex.as_deref().unwrap_or("")
        );
        let hits = index::query_index(&idx, term);
        let elapsed = start.elapsed();

        // Resolve hits back to actual file lines for display
        let mut matches = Vec::new();
        for (file_path, line_num) in &hits {
            let path = PathBuf::from(file_path);
            if let Ok(file) = std::fs::File::open(&path) {
                let reader = BufReader::new(file);
                if let Some(Ok(line)) = reader.lines().nth((*line_num as usize).saturating_sub(1)) {
                    matches.push(search::SearchMatch {
                        file_path: path,
                        line_number: *line_num as usize,
                        line_content: line,
                    });
                }
            }
        }

        output::print_results(
            &matches,
            &pattern,
            args.ignore_case,
            idx.files.len(),
            elapsed,
            &[],
            use_color,
        );

        if let Some(ref csv_path) = args.export {
            if let Err(e) = output::export_csv(&matches, csv_path) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        }

        return;
    }

    // --- Standard (non-indexed) search ---
    eprintln!("Scanning {} ...", dir.display());
    let scan_start = Instant::now();
    let (files, scan_warnings) = scanner::scan_directory(&dir, args.file_filter.as_deref());
    let scan_elapsed = scan_start.elapsed();
    eprintln!(
        "Found {} .txt files in {:.1}ms",
        files.len(),
        scan_elapsed.as_secs_f64() * 1000.0
    );

    for w in &scan_warnings {
        eprintln!("{}", w.yellow());
    }

    if files.is_empty() {
        eprintln!("No .txt files found in {}", dir.display());
        return;
    }

    let search_start = Instant::now();
    let results = search::parallel_search(&files, &pattern, args.ignore_case);
    let search_elapsed = search_start.elapsed();

    output::print_results(
        &results.matches,
        &pattern,
        args.ignore_case,
        results.files_scanned,
        search_elapsed,
        &results.warnings,
        use_color,
    );

    if let Some(ref csv_path) = args.export {
        if let Err(e) = output::export_csv(&results.matches, csv_path) {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}
