# STEP 2: Summary Statistics

## Objective

Track operation counts during merge and display a clear summary at completion, providing users with actionable feedback about what was processed.

## Implementation Details

### 1. Add Stats Struct (bin/merge-dirs-for-linux-limit.rs, after imports)

```rust
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
```

### 2. Modify Main Loop to Collect Stats (bin/merge-dirs-for-linux-limit.rs)

The current code uses `jdt::walk_dir` with a closure. We need to track stats across iterations:

```rust
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
        args.dst.clone()
    };

    // Track statistics
    let mut stats = Stats::default();

    // Traverse source directory and execute operations
    jdt::walk_dir(&src, |file_path| {
        // Compute relative path from source root
        let rel_path = file_path
            .strip_prefix(&src)
            .expect("file must be under src");

        // Resolve what action to take
        let action = resolve_action(&file_path, &dst, rel_path);

        // Update stats based on action type
        match &action {
            Action::Move { .. } => stats.moved += 1,
            Action::DeleteSrcOnly { .. } => stats.duplicates += 1,
            Action::Skip => stats.skipped += 1,
            Action::Error { .. } => stats.errors += 1,
        }

        // Execute the action
        if let Err(e) = execute_action(&action, &file_path, args.dry_run) {
            log::error!(
                "Failed to execute action for {}: {}",
                file_path.display(),
                e
            );
            // Note: Error during execution is separate from Action::Error
            // Action::Error = resolution failed, execution error = I/O failed
        }
    });

    // Print summary
    print_summary(&stats, args.dry_run);

    Ok(())
}
```

### 3. Add Summary Printing Function (bin/merge-dirs-for-linux-limit.rs)

```rust
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
```

## Code Snippets

### Complete Modified main.rs

```rust
use anyhow::Result;
use clap::Parser;
use rename_for_linux_limit::{execute_action, resolve_action, Action};
use std::path::PathBuf;

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
        args.dst.clone()
    };

    // Track statistics
    let mut stats = Stats::default();

    // Traverse source directory and execute operations
    jdt::walk_dir(&src, |file_path| {
        // Compute relative path from source root
        let rel_path = file_path
            .strip_prefix(&src)
            .expect("file must be under src");

        // Resolve what action to take
        let action = resolve_action(&file_path, &dst, rel_path);

        // Update stats based on action type
        match &action {
            Action::Move { .. } => stats.moved += 1,
            Action::DeleteSrcOnly { .. } => stats.duplicates += 1,
            Action::Skip => stats.skipped += 1,
            Action::Error { .. } => stats.errors += 1,
        }

        // Execute the action
        if let Err(e) = execute_action(&action, &file_path, args.dry_run) {
            log::error!(
                "Failed to execute action for {}: {}",
                file_path.display(),
                e
            );
        }
    });

    // Print summary
    print_summary(&stats, args.dry_run);

    Ok(())
}
```

## Impact Scope

| File | Function/Area | Change Type |
|------|---------------|-------------|
| `src/bin/merge-dirs-for-linux-limit.rs` | imports | Add `Action` to import |
| `src/bin/merge-dirs-for-linux-limit.rs` | `Stats` struct | New struct |
| `src/bin/merge-dirs-for-linux-limit.rs` | `print_summary()` | New function |
| `src/bin/merge-dirs-for-linux-limit.rs` | `main()` | Add stats tracking and summary call |

## Verification Method

### Manual Test

```bash
# Create test structure
mkdir -p /tmp/test_src/subdir
echo "content1" > /tmp/test_src/file1.txt
echo "content2" > /tmp/test_src/subdir/file2.txt
touch /tmp/test_src/.DS_Store
ln -s /tmp/test_src/file1.txt /tmp/test_src/link.txt  # symlink (skipped)

# Pre-create duplicate
mkdir -p /tmp/test_dst
echo "content1" > /tmp/test_dst/file1.txt  # Same content = duplicate

# Run dry-run
cargo run -- /tmp/test_src /tmp/test_dst --dry-run

# Expected output:
# [skip] /tmp/test_src/.DS_Store
# [skip] /tmp/test_src/link.txt
# [delete-src] /tmp/test_src/file1.txt -> /tmp/test_dst/file1.txt (duplicate)
# [move] /tmp/test_src/subdir/file2.txt -> /tmp/test_dst/subdir/file2.txt
#
# [dry-run] Summary: 1 moved, 1 duplicates, 2 skipped, 0 errors (4 total)
```

### Edge Cases to Verify

1. **Empty source directory**: Should print `Summary: 0 moved, 0 duplicates, 0 skipped, 0 errors (0 total)`

2. **All junk files**: Should show high skip count

3. **Execution errors**: If `execute_action` fails (I/O error), the action type still determines the stat category. Execution failures are logged separately.

## Design Notes

- **Stats in bin, not lib**: Statistics are a presentation concern. The library (`lib.rs`) focuses on action resolution and execution. The binary handles user-facing output.

- **Action import**: Need to add `Action` to the import from `rename_for_linux_limit` to match on it. This requires making `Action` public (already is: `pub enum Action`).

- **No separate "junk skipped" counter**: Per README, we use a single "skipped" counter combining symlinks and junk files. This keeps the summary simple. Users can use `RUST_LOG=trace` if they need detailed skip reasons.

- **Dry-run prefix**: Adding `[dry-run]` prefix to summary makes it clear the counts are projected, not actual operations performed.
