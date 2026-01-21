# Phase 1: CLI Scaffold + Dry-Run Foundation

## Core Context

**PROBLEM**: Need a CLI entry point for directory merging with dry-run capability built in from the start, using existing jdt library for directory traversal.

**SUCCESS**:
- User can run `merge-dirs-for-linux-limit <src> <dst> --dry-run`
- All files in source directory are enumerated
- Relative paths are correctly computed
- No files are modified in dry-run mode

**NOT DOING**:
- File content comparison (Phase 2)
- Actual file moves or copies (Phase 3)
- Filename shortening logic integration (Phase 3)
- Error handling polish (Phase 4)

**TRIGGER** (inherited from Sprint):
> 古い Mac を FUSE ベースの SFTP クライアント（Gnome の Files）でマウントして cp とか mv とかしたいときになんとか雑に名前を短くしたいので作った記憶がある
> /Volumes/Data -> /data のように２つの directory を deep に merge しつつ、名前が長かったら短くしつつ、データが同じかどうかを file size + CRC とかで確認しつつファイルを mac から linux に移したい
> 非常に大量かつ重要なアーカイブ的なものもありそうなので dry-run は必須

**PURPOSE**: Create the foundation CLI binary that Phase 2-4 will build upon. Dry-run is architectural (not optional) because important archives require safe preview.

**CONSTRAINTS**:
- Use `jdt::walk_dir` for recursive directory traversal
- Use clap with derive feature (consistent with existing main.rs)
- Create binary at `src/bin/merge-dirs-for-linux-limit.rs`
- Dry-run outputs planned operations only (no side effects)

## Technical Context

### Existing Code to Reference
- `src/main.rs`: Current single-file CLI using clap derive
- `src/lib.rs`: `new_filename()` function (not used in Phase 1, but Phase 3 will integrate)

### jdt Library Functions
- `jdt::walk_dir`: Recursive directory traversal
- `jdt::rename_file`: Cross-filesystem move (EXDEV-aware) - NOT used in Phase 1

### Key Decisions
- **Binary location**: `src/bin/merge-dirs-for-linux-limit.rs` (standard Cargo bin layout)
- **Argument structure**: `<src> <dst> --dry-run` (positional args + flag)
- **Output format**: Simple text listing of files to be processed

## STEPs

- [ ] STEP 1: Create CLI binary with argument parsing
- [ ] STEP 2: Implement directory traversal and relative path computation
- [ ] STEP 3: Add dry-run output formatting

## Review Status

- [ ] Initial review completed
- [ ] Review feedback applied
- [ ] Final verification completed
