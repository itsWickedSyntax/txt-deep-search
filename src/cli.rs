use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "txt-deep-search",
    about = "High-performance deep search tool for .txt files",
    version,
    after_help = "EXAMPLES:\n  \
        txt-deep-search /home/user/data --query \"search_term\"\n  \
        txt-deep-search /home/user/data --query \"pattern\" --ignore-case\n  \
        txt-deep-search /home/user/data --regex \"\\d{3}-\\d{4}\" --export results.csv\n  \
        txt-deep-search /home/user/data --query \"word\" --whole-word --threads 8\n  \
        txt-deep-search --index /home/user/data --query \"term\"\n  \
        txt-deep-search --reindex /home/user/data\n  \
        txt-deep-search --clear-index /home/user/data"
)]
pub struct Cli {
    /// Directory to search in
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,

    /// Search query (exact or partial match)
    #[arg(short, long)]
    pub query: Option<String>,

    /// Search using a regex pattern
    #[arg(short, long, conflicts_with = "query")]
    pub regex: Option<String>,

    /// Case-insensitive search
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Match whole words only
    #[arg(short, long)]
    pub whole_word: bool,

    /// Filter files by glob pattern (e.g. "report_*.txt")
    #[arg(short, long, value_name = "PATTERN")]
    pub file_filter: Option<String>,

    /// Number of threads for parallel processing
    #[arg(short, long, value_name = "NUM")]
    pub threads: Option<usize>,

    /// Export results to CSV file
    #[arg(short, long, value_name = "CSV_FILE")]
    pub export: Option<PathBuf>,

    /// Build/use inverted index for faster repeated searches
    #[arg(long)]
    pub index: bool,

    /// Rebuild the inverted index for the given directory
    #[arg(long, value_name = "DIRECTORY")]
    pub reindex: Option<PathBuf>,

    /// Clear the inverted index for the given directory
    #[arg(long, value_name = "DIRECTORY")]
    pub clear_index: Option<PathBuf>,
}

impl Cli {
    /// Returns the effective search pattern as a regex string.
    pub fn build_pattern(&self) -> Option<String> {
        let raw = if let Some(ref q) = self.query {
            regex::escape(q)
        } else if let Some(ref r) = self.regex {
            r.clone()
        } else {
            return None;
        };

        let with_word = if self.whole_word {
            format!(r"\b{}\b", raw)
        } else {
            raw
        };

        Some(with_word)
    }

}
