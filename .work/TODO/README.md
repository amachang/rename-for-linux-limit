# Phase 3: Move + Filename Shortening

## Core Context

**PROBLEM**: Phase 2 detects file status (new, duplicate, conflict) but doesn't execute any file operations. Need to implement the actual move logic with proper filename handling.

**SUCCESS**:
- Files are moved from src to dst with correct relative paths
- Long filenames (>255 bytes) are automatically shortened
- Conflicts get suffix (`.1`, `.2`, etc.) while staying <=255 bytes
- Duplicate files: src deleted, dst unchanged
- Source files deleted after successful move
- `--dry-run` shows planned operations without executing

**NOT DOING**:
- Comprehensive error handling (Phase 4)
- Summary statistics output (Phase 4)
- Symlink handling (Sprint scope exclusion - symlinks are skipped via `Action::Skip`)
- Permission/timestamp preservation (Sprint scope: best effort only)
- In-memory dry-run collision tracking (edge case, defer)

**TRIGGER** (inherited from Sprint):
> /Volumes/Data -> /data deep merge with name shortening and duplicate detection

**PURPOSE**: Implement the actual file migration logic that Phase 1-2 prepared for.

**CONSTRAINTS**:
- Reuse existing `new_candidate_filename()` logic from lib.rs
- Use `jdt::rename_file` for cross-filesystem moves
- Conflict suffix must respect 255-byte limit
- `--dry-run` must NOT modify any files

## Research Findings Summary

1. **`new_filename_impl` doesn't do duplicate detection** - just finds non-existing name. Need custom loop that handles: target doesn't exist (move), target exists with same content (delete src only), target exists with different content (retry with suffix).

2. **Directory creation in dry-run is acceptable** - for a personal tool, only file moves/deletes matter for safety preview. Creating directories early simplifies path resolution.

3. **`new_candidate_filename` is private** - need to make it `pub` or create a wrapper function to expose shortening logic.

4. **Config loading pattern** - `new_filename_impl` loads config via `jdt::project(crate_name!()).config::<Config>()`. Can be done once at startup.

## Phase 2 Completion Notes (Imported)

- **Implemented**: `files_have_same_size()`, `files_have_same_content()`, `compute_sha256()` in lib.rs; `determine_file_status()` in bin
- **Status labels**: `[new]`, `[duplicate]`, `[conflict (size)]`, `[conflict (content)]`, `[error]`
- **For Phase 3**: Replace string status with Action enum that captures actionable information

## STEPs

- [x] STEP 1: Expose shortening logic + resolve action
- [x] STEP 2: Execute actions

## Action Design

```rust
pub enum Action {
    Move { target: PathBuf },
    DeleteSrcOnly { target: PathBuf },  // target is the existing duplicate
    Skip,
    Error { message: String },
}
```

| Status | Action | Description |
|--------|--------|-------------|
| `new` | `Move { target }` | Copy to dst, delete src after verification |
| `duplicate` | `DeleteSrcOnly { target }` | Same content exists at dst, just remove src |
| `conflict (size/content)` | `Move { target }` | Apply `.N` suffix, then move |
| `error` | `Error { message }` | Log error, continue with next file |
