# STEP 1: Add comparison functions to lib.rs

## Overview

Add streaming file comparison functions to `src/lib.rs` for detecting duplicates and conflicts during directory merge operations.

## Files to Modify

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Add `sha2 = "0.10"` dependency |
| `src/lib.rs` | Add comparison functions and imports |

## Implementation Details

### 1. Add dependency to Cargo.toml

Add under `[dependencies]`:
```toml
sha2 = "0.10"
```

### 2. Add imports to lib.rs

Add at top of file (after existing imports):
```rust
use std::io;
use sha2::{Sha256, Digest};
```

### 3. Add comparison functions to lib.rs

Insert after the `normalize_str` function (line 290) and before `#[cfg(test)]`:

```rust
/// Compares file sizes. Returns true if both files exist and have the same size.
pub fn files_have_same_size(src: &Path, dst: &Path) -> io::Result<bool> {
    let src_meta = fs::metadata(src)?;
    let dst_meta = fs::metadata(dst)?;
    Ok(src_meta.len() == dst_meta.len())
}

/// Compares file contents using SHA256 hash.
/// Uses streaming to handle large files (50GB+).
/// Assumes caller has already verified sizes match (for performance).
pub fn files_have_same_content(src: &Path, dst: &Path) -> io::Result<bool> {
    let src_hash = compute_sha256(src)?;
    let dst_hash = compute_sha256(dst)?;
    Ok(src_hash == dst_hash)
}

fn compute_sha256(path: &Path) -> io::Result<[u8; 32]> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().into())
}
```

### 4. Add tests to lib.rs

Add inside `#[cfg(test)] mod tests`:

```rust
#[test]
fn test_files_have_same_size() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();

    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    let file3 = dir.path().join("file3.txt");

    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "hello").unwrap();
    fs::write(&file3, "hi").unwrap();

    assert!(files_have_same_size(&file1, &file2).unwrap());
    assert!(!files_have_same_size(&file1, &file3).unwrap());
}

#[test]
fn test_files_have_same_content() {
    let dir = tempfile::tempdir().unwrap();

    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    let file3 = dir.path().join("file3.txt");

    fs::write(&file1, "hello").unwrap();
    fs::write(&file2, "hello").unwrap();
    fs::write(&file3, "world").unwrap();  // same size, different content

    assert!(files_have_same_content(&file1, &file2).unwrap());
    assert!(!files_have_same_content(&file1, &file3).unwrap());
}

#[test]
fn test_compute_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.txt");
    fs::write(&file, "hello").unwrap();

    let hash = compute_sha256(&file).unwrap();
    // SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
    assert_eq!(
        hex::encode(hash),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}
```

**Note**: Tests use `tempfile` and `hex` crates. Add to `[dev-dependencies]`:
```toml
[dev-dependencies]
tempfile = "3"
hex = "0.4"
```

## Design Decisions

1. **Streaming hash**: Uses `io::copy` with `BufReader` for memory-efficient hashing of large files. This is mandatory per constraints (archives may be 50GB+).

2. **Separate functions**: `files_have_same_size` and `files_have_same_content` are separate to allow the caller to implement size-first comparison strategy.

3. **`io::Result` return type**: Matches standard library patterns. Caller decides how to handle I/O errors.

4. **Private `compute_sha256`**: Internal helper, not part of public API. Callers use `files_have_same_content` which handles the comparison.

5. **No caching**: Single-pass tool, caching adds unnecessary complexity (YAGNI).

## Completion Verification

1. `cargo build` succeeds
2. `cargo test` passes (including new tests)
3. `cargo clippy` has no warnings
4. Functions are exported from lib.rs (`pub fn`)

## Impact Scope

- **lib.rs**: Adds ~30 lines of code
- **Cargo.toml**: Adds 1 runtime dependency, 2 dev dependencies
- **No changes to existing functions** - purely additive
