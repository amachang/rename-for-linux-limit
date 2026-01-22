# Use walkdir with filter_entry() for Directory Traversal

## Decision

Use the `walkdir` crate with `filter_entry()` for directory traversal instead of `jdt::walk_dir`.

## Context

During Phase 4, the tool was getting stuck scanning `.Trash-1000` directories containing millions of deleted files. The Linux trash directory and other junk directories (`.DS_Store`, `lost+found`) needed to be skipped at traversal time, not filtered after enumeration.

The problem was not just performance - attempting to read certain directories like `lost+found` caused permission errors that disrupted the entire traversal.

## Options Considered

1. **Use `jdt::walk_dir` and filter files after enumeration**: Enumerate all files first, then filter out junk files from the resulting list
2. **Use `walkdir` crate with `filter_entry()`**: Prune directories during traversal using the callback-based filtering mechanism

## Rationale

Option 2 was chosen because:

- **Performance**: `filter_entry()` prevents descent into junk directories entirely. When `.Trash-1000` contains millions of files, skipping it at the directory level means zero file system calls into that subtree
- **Permission handling**: Avoids permission errors from attempting to read restricted directories like `lost+found` - they are skipped before any read is attempted
- **API limitation**: `jdt::walk_dir` does not expose equivalent directory-level pruning capability; it yields files after traversal, making early pruning impossible

## Consequences

**Positive**:
- Dramatically faster for sources with large junk directories (seconds instead of minutes)
- Avoids permission-denied errors on restricted directories
- Cleaner code - junk filtering logic is centralized in the filter callback

**Negative**:
- Additional dependency (walkdir crate) beyond the existing jdt dependency
- Slightly different API to learn compared to jdt utilities

## Date

2026-01-21 (Sprint: merge-dirs-for-linux-limit, Phase 4)
