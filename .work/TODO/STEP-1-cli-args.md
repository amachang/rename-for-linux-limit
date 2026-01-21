# STEP 1: CLI Binary with Argument Parsing and Validation

## Overview

Create `src/bin/merge-dirs-for-linux-limit.rs` with clap derive for CLI argument parsing. The binary accepts `<src>`, `<dst>`, and `--dry-run` arguments.

## Design

### File to Create

**Path**: `src/bin/merge-dirs-for-linux-limit.rs`

### Dependencies (already in Cargo.toml)

- `clap` with derive feature
- `anyhow` for Result type
- `std::path::PathBuf`
- `std::fs`

### Code Structure

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

    // Phase 1: Only dry-run is implemented
    // STEP 2 will add traversal logic here

    Ok(())
}
```

### Key Decisions

1. **Positional args for src/dst**: Matches typical Unix tool conventions (`cp`, `rsync`)

2. **`--dry-run` as flag**: Boolean flag, no value needed. In Phase 1 the tool only does dry-run anyway, but the flag establishes the interface for Phase 3.

3. **`canonicalize()` for src**: Required for reliable `strip_prefix` in STEP 2. Fails if path doesn't exist (desired behavior).

4. **Conditional canonicalize for dst**: Destination may not exist yet. Use as-is if non-existent.

5. **User input validation with bail**: User providing a non-directory is user error, not programmer error. Per CODING_CONVENTIONS, assertions are for programmer bugs only. User input errors use `if/else` with proper error handling (`anyhow::bail!`).

6. **No custom error types**: Following YAGNI - Phase 4 will polish error handling. For now, anyhow's automatic context is sufficient.

## Impact Scope

### Files Modified
- None (new file only)

### Files Created
- `src/bin/merge-dirs-for-linux-limit.rs`

### Cargo.toml
- No changes needed (binary auto-detected from `src/bin/` directory)

## Completion Verification

1. **Build succeeds**:
   ```bash
   cargo build --release
   ```

2. **Binary exists**:
   ```bash
   ls target/release/merge-dirs-for-linux-limit
   ```

3. **Help works**:
   ```bash
   ./target/release/merge-dirs-for-linux-limit --help
   ```
   Should show: src, dst arguments and --dry-run flag

4. **Validation works**:
   ```bash
   ./target/release/merge-dirs-for-linux-limit /nonexistent /tmp
   ```
   Should fail with canonicalize error

5. **Non-directory rejected**:
   ```bash
   ./target/release/merge-dirs-for-linux-limit /etc/passwd /tmp
   ```
   Should fail with error message

## Notes

- STEP 2 will add the traversal logic between args parsing and `Ok(())`
- The `dry_run` flag is parsed but not used in Phase 1 (all operations are dry-run)
- `env_logger::init()` follows existing pattern in `src/main.rs`
