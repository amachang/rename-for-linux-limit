use anyhow::Result;
use clap::Parser;
use rename_for_linux_limit::{execute_action, is_junk_path, resolve_action, Action};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "merge-dirs-for-linux-limit")]
#[command(about = "Merge directories with automatic filename shortening for Linux limits")]
struct Args {
    /// Source directory to merge from
    src: PathBuf,

    /// Destination directory to merge into
    dst: PathBuf,

    /// Show planned operations without executing
    #[arg(long)]
    dry_run: bool,
}

/// Statistics collected during merge operation
#[derive(Default)]
struct Stats {
    moved: usize,
    duplicates: usize,
    skipped: usize,
    errors: usize,
}

impl Stats {
    fn total(&self) -> usize {
        self.moved + self.duplicates + self.skipped + self.errors
    }
}

fn print_summary(stats: &Stats, dry_run: bool) {
    let prefix = if dry_run { "[dry-run] " } else { "" };

    println!();
    println!(
        "{}Summary: {} moved, {} duplicates, {} skipped, {} errors ({} total)",
        prefix,
        stats.moved,
        stats.duplicates,
        stats.skipped,
        stats.errors,
        stats.total()
    );
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    // Validate src exists and is directory
    let src = args.src.canonicalize()?;
    if !src.is_dir() {
        anyhow::bail!("Source must be a directory: {}", src.display());
    }

    // Canonicalize dst only if it exists, otherwise use as-is
    let dst = if args.dst.exists() {
        args.dst.canonicalize()?
    } else {
        args.dst
    };

    // Track statistics
    let mut stats = Stats::default();

    // Traverse source directory, skipping junk directories entirely
    let walker = WalkDir::new(&src).into_iter().filter_entry(|entry| {
        // Get path relative to src for junk check
        let rel_path = entry.path().strip_prefix(&src).unwrap_or(entry.path());
        !is_junk_path(rel_path)
    });

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Ignoring error: {}", e);
                continue;
            }
        };

        let file_path = entry.path();

        // Skip directories (we only process files)
        if file_path.is_dir() {
            continue;
        }

        // Compute relative path from source root
        let rel_path = file_path.strip_prefix(&src).expect("file must be under src");

        // Resolve what action to take
        let action = resolve_action(file_path, &dst, rel_path);

        // Execute the action and update stats based on outcome
        match execute_action(&action, file_path, args.dry_run) {
            Ok(()) => match &action {
                Action::Move { .. } => stats.moved += 1,
                Action::DeleteSrcOnly { .. } => stats.duplicates += 1,
                Action::Skip => stats.skipped += 1,
                Action::Error { .. } => stats.errors += 1,
            },
            Err(e) => {
                log::error!("Failed to execute action for {}: {}", file_path.display(), e);
                stats.errors += 1;
            }
        }
    }

    // Print summary
    print_summary(&stats, args.dry_run);

    Ok(())
}
