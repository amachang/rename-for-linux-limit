# Sprint: merge-dirs-for-linux-limit

**Status**: Active
**Start Date**: 2026-01-20
**Target Date**: TBD

## Core Context

**PROBLEM**: Migrating entire directories from Mac to Linux hits Linux's 255-byte filename limit. The existing `rename-for-linux-limit` tool handles single files only—it cannot recursively process directories, merge folder structures, or detect duplicate files.

**SUCCESS**:
- `merge-dirs-for-linux-limit <src> <dst>` recursively merges directories
- Long filenames are automatically shortened to 255-byte limit
- Duplicate detection via file size + SHA256 hash
- `--dry-run` for safe preview (MANDATORY for important archives)
- Move behavior: delete source file after successful copy

**NOT DOING**:
- GUI / TUI (CLI only)
- Bidirectional sync (src → dst one-way only)
- Symlink special handling (skip symlinks)
- Full permission/timestamp preservation (best effort only)
- Empty source directory cleanup (leave empty dirs in source after move)

**TRIGGER**:
> 古い Mac を FUSE ベースの SFTP クライアント（Gnome の Files）でマウントして cp とか mv とかしたいときになんとか雑に名前を短くしたいので作った記憶がある
> /Volumes/Data -> /data のように２つの directory を deep に merge しつつ、名前が長かったら短くしつつ、データが同じかどうかを file size + CRC とかで確認しつつファイルを mac から linux に移したい
> 非常に大量かつ重要なアーカイブ的なものもありそうなので dry-run は必須

**PURPOSE**: Safe data migration tool from Mac to Linux with automatic filename length handling.

**CONSTRAINTS**:
- dry-run is mandatory (important archives)
- Reuse existing `new_filename` logic from lib.rs
- Use `jdt::walk_dir` and `jdt::rename_file`
- Conflict suffix must also respect 255-byte limit

## Sprint Goal

Deliver a CLI command that enables safe, one-way directory migration from Mac to Linux. The tool recursively merges source into destination, automatically shortens filenames exceeding Linux's 255-byte limit, detects duplicates via content comparison, and provides a mandatory dry-run mode to preview all operations before execution.

## User Stories

### Must (Implement Now)

**User Story 1**: Recursive Directory Merge

As a user migrating data from Mac to Linux,
I want to recursively merge a source directory into a destination directory,
So that I can migrate my entire folder structure in one operation.

**Acceptance Criteria**:
- [ ] User can run `merge-dirs-for-linux-limit <src> <dst>`
- [ ] Nested directories are traversed correctly
- [ ] Relative path structure is preserved in destination
- [ ] Symlinks are skipped (not followed, not copied)

**Phases**: Phase 1

---

**User Story 2**: Automatic Filename Shortening with Conflict Resolution

As a user with files that have long names,
I want filenames automatically shortened to fit Linux's 255-byte limit,
So that the migration doesn't fail due to filename restrictions.

**Acceptance Criteria**:
- [ ] Filenames exceeding 255 bytes are automatically truncated
- [ ] Existing `new_filename` logic from lib.rs is reused
- [ ] Conflicts (same name exists) are resolved with suffix numbering (_1, _2, etc.)
- [ ] Suffix addition also respects 255-byte limit (truncate base if needed)
- [ ] Shortened/renamed files are logged

**Phases**: Phase 3

---

**User Story 3**: Duplicate Detection

As a user with potentially duplicate files,
I want the tool to detect identical files by size + hash,
So that I don't waste space copying duplicates.

**Acceptance Criteria**:
- [ ] Files are compared by size first (fast path)
- [ ] Only size-matching files are hashed (SHA256)
- [ ] Identical content: skip move, delete source only
- [ ] Different content: treat as conflict, apply suffix
- [ ] Detection results are logged

**Phases**: Phase 2, Phase 3

---

**User Story 4**: Safe Preview (Dry-Run)

As a user with important archives,
I want to preview all operations before execution,
So that I can verify the migration plan before committing.

**Acceptance Criteria**:
- [ ] `--dry-run` flag skips all write operations
- [ ] Planned operations are displayed (move, skip, conflict)
- [ ] Summary shows file count and total bytes
- [ ] Dry-run is the recommended first step

**Phases**: Phase 1, Phase 4

---

### Later (Deferred for Future Consideration)

- **Progress Bar**: Real-time progress display with ETA
- **Symlink Handling**: Option to follow or recreate symlinks
- **Permission Preservation**: Maintain file permissions and timestamps
- **Parallel Processing**: Multi-threaded file operations for speed

## Branch Naming Convention

Feature branches follow the pattern:
```
feature/<function>-phase<N>-<description>
```

Examples:
- `feature/merge-phase1-scaffold-dryrun`
- `feature/merge-phase2-compare-hash`
- `feature/merge-phase3-move-shorten`
- `feature/merge-phase4-polish`

## Phases

### Phase 1: CLI Scaffold + Dry-Run Foundation

**Branch**: `feature/merge-phase1-scaffold-dryrun`

**User Stories**: Addresses User Story 1 (partial), User Story 4 (partial)

**Problem Statement**:
Need a CLI entry point with dry-run capability built in from the start, and directory traversal using existing jdt library.

**Solution Approach**:
Create new binary at `src/bin/merge-dirs-for-linux-limit.rs`. Use clap for argument parsing (`<src>`, `<dst>`, `--dry-run`). Integrate `jdt::walk_dir` for recursive file enumeration. Establish dry-run as architectural foundation—all subsequent phases check this flag before any write operation.

**Success Criteria**:
- User can run `merge-dirs-for-linux-limit <src> <dst> --dry-run`
- All files in source directory are enumerated
- Relative paths are correctly computed
- No files are modified in dry-run mode

**Status**: ✅ Complete

#### Phase 1 Completion Notes

- **Implemented**: `src/bin/merge-dirs-for-linux-limit.rs` (51 lines)
- **Key pattern**: `canonicalize()` for src (required for reliable `strip_prefix`), conditional for dst (may not exist yet)
- **`jdt::walk_dir`** returns files only (directories traversed but not yielded) - relevant for Phase 2+ output formatting
- **`--dry-run` flag** parsed but unused in Phase 1 - establishes interface for Phase 3
- **Deferred**: Symlink filtering (Phase 3), `new_filename()` integration (Phase 3)

---

### Phase 2: Content Comparison (Size + Hash)

**Branch**: `feature/merge-phase2-compare-hash`

**User Stories**: Addresses User Story 3 (partial)

**Problem Statement**:
Need to detect duplicate files across source and destination to avoid unnecessary copies and handle conflicts.

**Solution Approach**:
Add `sha2` crate for SHA256 hashing. Implement size-first comparison (fast path). Hash only when sizes match. Note: `jdt::eq_files` uses inode comparison and does NOT work for cross-filesystem (Mac→Linux) scenarios.

**Success Criteria**:
- Size comparison works correctly
- SHA256 hash is computed for size-matching files
- Identical files are detected and marked for skip
- Different-content same-name files are detected as conflicts

---

### Phase 3: Move + Filename Shortening

**Branch**: `feature/merge-phase3-move-shorten`

**User Stories**: Addresses User Story 1 (completion), User Story 2, User Story 3 (completion)

**Problem Statement**:
Execute the actual file migration with proper filename handling and conflict resolution.

**Solution Approach**:
Reuse `new_filename()` from lib.rs for shortening. Use `jdt::rename_file` for cross-filesystem moves (handles EXDEV). Important: when adding suffix, ensure result is still ≤255 bytes—truncate base filename if needed to make room for suffix. Delete source file only after successful copy verification.

**Success Criteria**:
- Files are moved to destination with correct relative paths
- Long filenames are shortened to ≤255 bytes
- Conflicts get suffix (_1, _2, etc.) while staying ≤255 bytes
- Identical files: source deleted, destination unchanged
- Source files are deleted after successful move

---

### Phase 4: Error Handling + Polish

**Branch**: `feature/merge-phase4-polish`

**User Stories**: Addresses User Story 4 (completion), all stories (polish)

**Problem Statement**:
Robust error handling, clear logging, and user-friendly output.

**Solution Approach**:
Handle permission errors gracefully (log and continue). Provide clear operation summary (moved, skipped, conflicts, errors). Ensure dry-run output matches actual execution behavior.

**Success Criteria**:
- Permission errors are logged but don't abort entire operation
- Clear summary at end: X moved, Y skipped, Z conflicts, W errors
- Dry-run output accurately predicts actual behavior
- All operations are logged at appropriate verbosity levels

---

## Technical Notes

### Dependencies to Add
- `sha2` crate for SHA256 hashing

### Code Reuse
- `lib.rs::new_filename()` - filename shortening with suffix handling
- `jdt::walk_dir` - recursive directory traversal
- `jdt::rename_file` - cross-filesystem move (EXDEV-aware)

### Do NOT Use
- `jdt::eq_files` - uses inode comparison, doesn't work cross-filesystem

### Move Verification
- `jdt::rename_file` handles EXDEV by copy+delete internally
- For critical archives, verify copy success (size match) before source deletion
- If hash was computed for duplicate check, optionally verify destination hash matches

### Known Issues
- `new_filename()` calls `fs::create_dir_all()` internally - need to handle for dry-run mode
  - Solution options: (a) check dry-run flag before calling, (b) use `new_filename_impl()` directly with custom callbacks, (c) refactor to separate pure name calculation from directory creation
- `new_filename()` uses `clap::crate_name!()` for config loading - new binary may need explicit project name parameter
- Existing suffix logic (`.1`, `.2`) in `new_candidate_filename()` already handles 255-byte limit - reuse this rather than implementing new logic

### Suffix + 255-byte Limit
When adding conflict suffix (_1, _2, etc.), the result may exceed 255 bytes. Solution: truncate base filename to make room for suffix before adding it. Example:
- 255-byte filename + `_1` = 258 bytes (NG)
- Truncate to 252 bytes + `_1` = 255 bytes (OK)

## Deferred Stories

Stories marked "Later" above will be revisited at sprint-complete:
- Evaluate for next sprint inclusion
- Archive if no longer relevant
- Refine and re-prioritize if scope becomes clearer

## References

**Related Documentation**:
- Existing tool: `src/main.rs` (single-file rename)
- Core logic: `src/lib.rs` (new_filename function)

**Dependencies**:
- jdt library (private): `ssh://git@github.com/amachang/jdt.git`
- Phase 2 depends on Phase 1 (CLI foundation)
- Phase 3 depends on Phase 2 (comparison logic)
- Phase 4 depends on Phase 3 (needs working implementation to polish)

## Notes

- This is a personal migration tool, not a production service
- Safety over speed: dry-run is mandatory for important archives
- Single-threaded for correctness; parallelism deferred
