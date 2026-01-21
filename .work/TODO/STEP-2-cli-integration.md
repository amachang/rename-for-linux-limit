# STEP 2: Integrate comparison into CLI

## Overview

Modify `merge-dirs-for-linux-limit` binary to use comparison functions and output status labels for each file.

## Files to Modify

| File | Change Type |
|------|-------------|
| `src/bin/merge-dirs-for-linux-limit.rs` | Add comparison logic and status output |

## Implementation Details

### 1. Add import for lib functions

At top of file, add:
```rust
use rename_for_linux_limit::{files_have_same_size, files_have_same_content};
```

### 2. Replace the walk_dir callback

Replace the entire `jdt::walk_dir` block (lines 39-48):

```rust
// Traverse source directory and print planned operations
jdt::walk_dir(&src, |file_path| {
    // Compute relative path from source root
    let rel_path = file_path.strip_prefix(&src).expect("file must be under src");

    // Compute target path
    let target_path = dst.join(rel_path);

    // Determine file status
    let status = determine_file_status(file_path, &target_path);

    // Output planned operation with status
    println!("[{}] {} -> {}", status, file_path.display(), target_path.display());
});
```

### 3. Add status determination function

Add after `main()`:

```rust
fn determine_file_status(src: &Path, dst: &Path) -> &'static str {
    if !dst.exists() {
        return "new";
    }

    // Size comparison first (fast path)
    match files_have_same_size(src, dst) {
        Ok(true) => {
            // Same size - need to compare content
            match files_have_same_content(src, dst) {
                Ok(true) => "duplicate",
                Ok(false) => "conflict (content)",
                Err(e) => {
                    log::warn!("Hash comparison failed: {} vs {}: {}", src.display(), dst.display(), e);
                    "error"
                }
            }
        }
        Ok(false) => "conflict (size)",
        Err(e) => {
            log::warn!("Size comparison failed: {} vs {}: {}", src.display(), dst.display(), e);
            "error"
        }
    }
}
```

### 4. Add Path import

Update imports at top of file:
```rust
use std::path::{Path, PathBuf};
```

## Complete Modified File

For clarity, here is the complete file after modifications:

```rust
use std::path::{Path, PathBuf};
use clap::Parser;
use anyhow::Result;
use rename_for_linux_limit::{files_have_same_size, files_have_same_content};

#[derive(Parser, Debug)]
#[command(name = "merge-dirs-for-linux-limit")]
#[command(about = "Preview directory merge operations for Linux filename limits")]
struct Args {
    /// Source directory to merge from
    src: PathBuf,

    /// Destination directory to merge into
    dst: PathBuf,

    /// Show planned operations without executing
    #[arg(long)]
    dry_run: bool,
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

    // Traverse source directory and print planned operations
    jdt::walk_dir(&src, |file_path| {
        // Compute relative path from source root
        let rel_path = file_path.strip_prefix(&src).expect("file must be under src");

        // Compute target path
        let target_path = dst.join(rel_path);

        // Determine file status
        let status = determine_file_status(file_path, &target_path);

        // Output planned operation with status
        println!("[{}] {} -> {}", status, file_path.display(), target_path.display());
    });

    Ok(())
}

fn determine_file_status(src: &Path, dst: &Path) -> &'static str {
    if !dst.exists() {
        return "new";
    }

    // Size comparison first (fast path)
    match files_have_same_size(src, dst) {
        Ok(true) => {
            // Same size - need to compare content
            match files_have_same_content(src, dst) {
                Ok(true) => "duplicate",
                Ok(false) => "conflict (content)",
                Err(e) => {
                    log::warn!("Hash comparison failed: {} vs {}: {}", src.display(), dst.display(), e);
                    "error"
                }
            }
        }
        Ok(false) => "conflict (size)",
        Err(e) => {
            log::warn!("Size comparison failed: {} vs {}: {}", src.display(), dst.display(), e);
            "error"
        }
    }
}
```

## Design Decisions

1. **`&'static str` return type**: Status strings are compile-time constants. No allocation needed.

2. **Size comparison first**: Fast path optimization. Hash is only computed when sizes match.

3. **Error as status**: I/O errors during comparison are logged and reported as "error" status. This allows the user to see which files had issues without stopping the entire scan.

4. **Early return for new files**: Most common case in migration - avoids unnecessary comparison work.

5. **Log warnings on errors**: Uses existing `env_logger` infrastructure. User can enable with `RUST_LOG=warn`.

## Output Examples

```
[new] /src/path/file.txt -> /dst/path/file.txt
[duplicate] /src/path/same.txt -> /dst/path/same.txt
[conflict (size)] /src/path/diff.txt -> /dst/path/diff.txt
[conflict (content)] /src/path/diff2.txt -> /dst/path/diff2.txt
[error] /src/path/unreadable.txt -> /dst/path/unreadable.txt
```

## Completion Verification

1. `cargo build --release` succeeds
2. `cargo clippy` has no warnings
3. Manual test with sample directories:
   - Create src/ with test files
   - Create dst/ with mix of: missing files, identical files, same-size-different-content files, different-size files
   - Run `./target/release/merge-dirs-for-linux-limit src/ dst/ --dry-run`
   - Verify each file shows correct status label

## Impact Scope

- **merge-dirs-for-linux-limit.rs**: Modifies ~15 lines, adds ~25 lines
- **No changes to lib.rs** - uses functions from STEP 1
- **No new dependencies** - uses existing imports

## Dependencies

This STEP depends on STEP 1 being complete (comparison functions must exist in lib.rs).
