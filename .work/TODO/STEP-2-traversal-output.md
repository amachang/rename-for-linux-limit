# STEP 2: Directory Traversal with Output

## Overview

Add directory traversal using `jdt::walk_dir` to enumerate all files in source directory, compute relative paths, and print planned operations.

## Design

### File to Modify

**Path**: `src/bin/merge-dirs-for-linux-limit.rs`

### Code Changes

Add traversal logic after argument validation in `main()`:

```rust
use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

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

        // Output planned operation
        println!("{} -> {}", file_path.display(), target_path.display());
    });

    Ok(())
}
```

### Key Decisions

1. **`strip_prefix` with expect**: Since `walk_dir` only returns files under the source directory, and we canonicalized both paths, `strip_prefix` cannot fail. Using `expect` per CODING_CONVENTIONS - this would be a bug in our logic.

2. **Simple text output**: `{src_path} -> {dst_path}` format. No JSON, no structured output per NOT_DOING.

3. **No return value from closure**: The closure just prints. `walk_dir` returns `Vec<()>` which we ignore.

4. **Absolute paths in output**: Both source and target paths are absolute for clarity. Source is canonicalized, target is `dst.join(rel_path)`.

5. **No file counting or progress**: Per NOT_DOING, progress reporting is deferred.

## Impact Scope

### Files Modified
- `src/bin/merge-dirs-for-linux-limit.rs` (add traversal logic)

### Dependencies Used
- `jdt::walk_dir` - already in Cargo.toml

## Example Output

Given:
```
/home/user/source/
  file1.txt
  subdir/
    file2.txt
```

Running:
```bash
merge-dirs-for-linux-limit /home/user/source /mnt/dest --dry-run
```

Output:
```
/home/user/source/file1.txt -> /mnt/dest/file1.txt
/home/user/source/subdir/file2.txt -> /mnt/dest/subdir/file2.txt
```

## Completion Verification

1. **Build succeeds**:
   ```bash
   cargo build --release
   ```

2. **Create test directory structure**:
   ```bash
   mkdir -p /tmp/merge-test/src/subdir
   touch /tmp/merge-test/src/file1.txt
   touch /tmp/merge-test/src/subdir/file2.txt
   ```

3. **Run dry-run**:
   ```bash
   ./target/release/merge-dirs-for-linux-limit /tmp/merge-test/src /tmp/merge-test/dst --dry-run
   ```

   Expected output (paths canonicalized):
   ```
   /tmp/merge-test/src/file1.txt -> /tmp/merge-test/dst/file1.txt
   /tmp/merge-test/src/subdir/file2.txt -> /tmp/merge-test/dst/subdir/file2.txt
   ```

4. **Verify no files created**:
   ```bash
   ls /tmp/merge-test/dst
   ```
   Should fail (directory doesn't exist) - confirming dry-run did nothing

5. **Cleanup**:
   ```bash
   rm -rf /tmp/merge-test
   ```

## Notes

- `jdt::walk_dir` silently skips unreadable files/directories with logging
- Symlinks are followed by `walk_dir` (deferred to Phase 3 for filtering)
- Empty directories are not listed (walk_dir returns files only)
- The `--dry-run` flag is accepted but all operations are already dry-run in Phase 1
- **Relative dst path handling**: If dst doesn't exist and is specified as a relative path, output will show relative target paths. This is acceptable for Phase 1 dry-run preview.
  - Example: `./target/release/merge-dirs-for-linux-limit /tmp/src relative-dst --dry-run` will output `/tmp/src/file.txt -> relative-dst/file.txt`
