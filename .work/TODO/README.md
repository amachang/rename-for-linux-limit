# Phase 3: Move + Filename Shortening

## Core Context

**PROBLEM**: Phase 2 detects file status (new, duplicate, conflict) but doesn't execute any file operations. Need to implement the actual move logic with proper filename handling.

**SUCCESS**:
- Files are moved from src to dst with correct relative paths
- Long filenames (>255 bytes) are automatically shortened
- Conflicts get suffix (`.1`, `.2`, etc.) while staying ≤255 bytes
- Duplicate files: src deleted, dst unchanged
- Source files deleted after successful move
- `--dry-run` shows planned operations without executing

**NOT DOING**:
- Comprehensive error handling (Phase 4)
- Summary statistics output (Phase 4)
- Symlink handling (Sprint scope exclusion)
- Permission/timestamp preservation (Sprint scope: best effort only)

**TRIGGER** (inherited from Sprint):
> 古い Mac を FUSE ベースの SFTP クライアント（Gnome の Files）でマウントして cp とか mv とかしたいときになんとか雑に名前を短くしたいので作った記憶がある
> /Volumes/Data -> /data のように２つの directory を deep に merge しつつ、名前が長かったら短くしつつ、データが同じかどうかを file size + CRC とかで確認しつつファイルを mac から linux に移したい
> 非常に大量かつ重要なアーカイブ的なものもありそうなので dry-run は必須

**PURPOSE**: Implement the actual file migration logic that Phase 1-2 prepared for.

**CONSTRAINTS**:
- Reuse existing `new_filename()` logic from lib.rs
- Use `jdt::rename_file` for cross-filesystem moves
- Conflict suffix must respect 255-byte limit
- `--dry-run` must NOT modify any files

## Phase 2 Completion Notes (Imported)

- **Implemented**: `files_have_same_size()`, `files_have_same_content()`, `compute_sha256()` in lib.rs; `determine_file_status()` in bin
- **Status labels**: `[new]`, `[duplicate]`, `[conflict (size)]`, `[conflict (content)]`, `[error]`
- **For Phase 3**: Map status to action - new: move, duplicate: delete src only, conflict: apply suffix then move

## Known Issues from Sprint Planning

1. **`new_filename()` calls `fs::create_dir_all()` internally**
   - Solution: Use `new_filename_impl()` directly with custom `check_file_existence` callback
   - For dry-run: Return calculated name without creating directories
   - For execution: Let it create directories as normal

2. **`new_filename()` uses `clap::crate_name!()` for config loading**
   - The new binary is in the same crate, so `crate_name!()` returns "rename-for-linux-limit"
   - Should work correctly without modification

3. **Suffix format is `.N` not `_N`**
   - Existing `new_candidate_filename()` produces: `filename.1.txt`, `filename.2.txt`
   - Already handles 255-byte limit by truncating base filename

## Status-to-Action Mapping

| Status | Action | Description |
|--------|--------|-------------|
| `new` | Move | Copy to dst, delete src after verification |
| `duplicate` | Delete src | Same content exists at dst, just remove src |
| `conflict (size)` | Move with suffix | Apply `.N` suffix, then move |
| `conflict (content)` | Move with suffix | Apply `.N` suffix, then move |
| `error` | Skip + Log | Log error, continue with next file |

## Implementation Strategy

### Approach: Direct Integration

1. **Extend `determine_file_status()`** to return actionable information (not just display string)
2. **Implement `execute_action()`** that performs the actual file operation based on status
3. **Handle dry-run** by checking flag before any write operation
4. **Use existing functions**:
   - `new_filename_impl()` for filename calculation + conflict resolution
   - `jdt::rename_file()` for cross-filesystem move

### Key Decisions

1. **Return type for status**: Create an enum `FileStatus` instead of `&'static str` for type safety
2. **Directory creation**: Create parent directories before move operation
3. **Verification**: After move, verify destination exists and has expected size
4. **Error handling**: Log errors but continue processing (fail-soft for this phase)

## STEPs

(To be defined after RESEARCH stage)

- [ ] STEP 1: TBD
- [ ] STEP 2: TBD
- [ ] STEP 3: TBD
