# Canonicalize Source/Destination Path Asymmetry

## Problem

`fs::canonicalize()` behaves asymmetrically for source and destination paths - it requires the path to exist.

## Symptoms

- `strip_prefix()` fails with cryptic errors when source path contains symlinks
- "No such file or directory" errors when trying to canonicalize destination that does not exist yet
- Path operations produce unexpected results due to unresolved symlinks

## Root Cause

`fs::canonicalize()` resolves symlinks and converts relative paths to absolute paths, but it requires the path to exist on the filesystem. This creates an asymmetry:

- **Source path**: Must exist (we are reading from it), so canonicalization works
- **Destination path**: May not exist yet (we are creating it), so canonicalization fails

Additionally, `strip_prefix()` requires the prefix to exactly match the path. If the source contains symlinks that are not resolved, `strip_prefix()` will fail even when the paths logically match.

## Solution

Handle source and destination paths asymmetrically:

```rust
let src = args.src.canonicalize()?;  // Required - strip_prefix needs resolved path
let dst = if args.dst.exists() {
    args.dst.canonicalize()?
} else {
    args.dst  // Cannot canonicalize non-existent path
};
```

For the source path, canonicalization is essential for reliable `strip_prefix()` operations. For the destination path, only canonicalize if it exists.

## Prevention

When working with path pairs where one must exist (source) and one may not (destination):

1. Always canonicalize the source path to ensure consistent prefix matching
2. Check existence before canonicalizing the destination path
3. Document this asymmetry in function signatures when accepting path pairs
4. Consider using `dunce::canonicalize()` on Windows to avoid UNC path issues
