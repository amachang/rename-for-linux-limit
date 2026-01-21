# Phase 2: Content Comparison (Size + Hash)

**Branch**: `feature/merge-phase2-compare-hash`
**User Stories**: Addresses User Story 3 (Duplicate Detection) - partial

## Core Context

**PROBLEM**: Need to detect duplicate files across source and destination to avoid unnecessary copies and handle conflicts during Mac-to-Linux migration.

**SUCCESS**:
- Size comparison works correctly
- SHA256 hash is computed for size-matching files only (fast path optimization)
- Identical files are detected and marked for skip
- Different-content same-name files are detected as conflicts

**NOT DOING**:
- Actual file move/copy operations (Phase 3)
- Filename shortening logic (Phase 3)
- Source file deletion (Phase 3)
- Error handling polish (Phase 4)

**TRIGGER** (inherited from Sprint):
> 古い Mac を FUSE ベースの SFTP クライアント（Gnome の Files）でマウントして cp とか mv とかしたいときになんとか雑に名前を短くしたいので作った記憶がある
> /Volumes/Data -> /data のように２つの directory を deep に merge しつつ、名前が長かったら短くしつつ、データが同じかどうかを file size + CRC とかで確認しつつファイルを mac から linux に移したい
> 非常に大量かつ重要なアーカイブ的なものもありそうなので dry-run は必須

**PURPOSE**: Implement content comparison logic to identify duplicates and conflicts before file operations.

**CONSTRAINTS**:
- Do NOT use `jdt::eq_files` (uses inode comparison, doesn't work cross-filesystem)
- Add `sha2` crate for SHA256 hashing
- Size-first comparison for performance (hash only when sizes match)
- This is a personal migration tool, not production service

## Technical Context

### Dependencies
- Add `sha2` crate for SHA256 hashing

### Phase 1 Foundation (Already Implemented)
- CLI with `<src>`, `<dst>`, `--dry-run` arguments
- Directory traversal using `jdt::walk_dir`
- Relative path computation from source root

### Key Files
- `src/bin/merge-dirs-for-linux-limit.rs` - CLI binary (51 lines from Phase 1)
- `src/lib.rs` - Core library (new_filename logic for Phase 3)

## STEPs

- [ ] STEP 1: Add sha2 dependency and implement comparison functions
- [ ] STEP 2: Integrate comparison into CLI and output results
- [ ] Initial review with multiple agents
- [ ] Apply review feedback (iteration 1)
- [ ] Second review with multiple agents
- [ ] Apply review feedback if needed (iteration 2)
- [ ] Final validation

## Status

**Current**: Planning (Dry Coding)
