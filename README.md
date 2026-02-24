# txt-deep-search

A high-performance CLI tool for deep-searching `.txt` files. Built in Rust with multi-threaded parallel processing, memory-mapped I/O for large files, ANSI-highlighted output, and an optional inverted index for instant repeated searches.

## Features

- **Recursive directory scanning** — automatically walks subdirectories, filters `.txt` files, skips binary files
- **Multiple search modes** — exact match, partial match, regex, case-insensitive, whole-word
- **Multi-threaded** — parallel file processing via rayon (configurable thread count)
- **Memory efficient** — streaming line-by-line for normal files, mmap for files >10 MB
- **Rich output** — ANSI color-highlighted matches, file path, line number
- **Progress bar** — real-time progress feedback during scan and search
- **CSV export** — export results to CSV for further analysis
- **Inverted index** — build a persistent index for instant repeated searches
- **Robust error handling** — gracefully skips permission errors, unreadable files, malformed encoding

## Requirements

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Linux (Ubuntu/Debian-based recommended)

## Build

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release

# Or use make
make release
```

## Install

```bash
# Install to /usr/local/bin (requires sudo)
sudo make install

# Or install via cargo
cargo install --path .

# Or copy manually
cp target/release/txt-deep-search ~/.local/bin/
```

## Uninstall

```bash
sudo make uninstall
# or
cargo uninstall txt-deep-search
```

## Usage

### Basic search

```bash
txt-deep-search /home/user/data --query "search_term"
```

### Case-insensitive search

```bash
txt-deep-search /home/user/data --query "Search_Term" --ignore-case
```

### Regex search

```bash
txt-deep-search /home/user/data --regex "\d{3}-\d{4}"
```

### Whole-word matching

```bash
txt-deep-search /home/user/data --query "word" --whole-word
```

### Filter by filename pattern

```bash
txt-deep-search /home/user/data --query "error" --file-filter "log_*.txt"
```

### Set thread count

```bash
txt-deep-search /home/user/data --query "term" --threads 8
```

### Export to CSV

```bash
txt-deep-search /home/user/data --query "term" --export results.csv
```

### Indexed search (fast repeated queries)

```bash
# First search builds the index automatically
txt-deep-search /home/user/data --index --query "term"

# Subsequent searches use the pre-built index (instant)
txt-deep-search /home/user/data --index --query "another_term"

# Manually rebuild the index
txt-deep-search --reindex /home/user/data

# Clear the index
txt-deep-search --clear-index /home/user/data
```

## Output Format

Each match is displayed as:

```
/path/to/file.txt:42  The line containing the highlighted match
```

- File path in green
- Line number in yellow
- Matched term in bold red

A summary is printed at the end:

```
Results: 15 matches across 2000 files scanned in 45.3ms
```

## Benchmarking

```bash
make bench
```

This creates 2000 test files and compares `txt-deep-search` against `grep -rn`.

## Project Structure

```
txt-deep-search/
├── Cargo.toml          # Dependencies and build config
├── Makefile            # Build/install/bench targets
├── README.md           # This file
└── src/
    ├── main.rs         # Entry point, orchestration
    ├── cli.rs          # CLI argument parsing (clap)
    ├── scanner.rs      # Recursive directory scanning
    ├── search.rs       # Core search engine (parallel, mmap)
    ├── index.rs        # Inverted index (build/query/clear)
    └── output.rs       # Output formatting, CSV export
```

## License

MIT
