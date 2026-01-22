# Cross-Filesystem Inode Comparison Failure

## Problem

`jdt::eq_files` uses inode comparison which fails for cross-filesystem file equality checks.

## Symptoms

- Duplicate detection returns false for identical files across different filesystems
- Files on Mac source and Linux destination are never detected as duplicates
- Copy operations proceed even when destination already has an identical file

## Root Cause

`jdt::eq_files` compares file inodes, which are filesystem-local identifiers. Two files on different filesystems will always have different inodes even if their content is identical. This is by design - inodes are only unique within a single filesystem.

When migrating files from Mac to Linux (different filesystems), inode-based comparison will never find matches.

## Solution

Use size + SHA256 hash comparison instead:

```rust
fn files_have_same_content(path1: &Path, path2: &Path) -> io::Result<bool> {
    if !files_have_same_size(path1, path2)? {
        return Ok(false);
    }
    let hash1 = compute_sha256(path1)?;
    let hash2 = compute_sha256(path2)?;
    Ok(hash1 == hash2)
}
```

The size check provides a fast early-exit for most non-matching files, avoiding expensive hash computation.

## Prevention

When cross-filesystem operations are in scope, always verify file comparison functions use content-based comparison, not metadata-based. Ask these questions:

1. Does the comparison function rely on inodes, device IDs, or other filesystem-specific metadata?
2. Will source and destination ever be on different filesystems?
3. If yes to both, switch to content-based comparison (hash + size).
