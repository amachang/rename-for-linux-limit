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
- `src/lib.rs`: `new_filename()` function (NOT used in Phase 1 - has side effects)

### jdt Library Functions (Research Findings)

**jdt::walk_dir API**:
```rust
pub fn walk_dir<R>(dir: impl AsRef<Path>, f: impl FnMut(PathBuf) -> R) -> Vec<R>
```
- Returns **files only** (directories are traversed but not returned)
- Gracefully skips errors with logging (continues on permission errors)
- Uses `path.is_dir()` which follows symlinks (defer handling to Phase 3)
- Stack-based non-recursive implementation (safe for deep directories)

### Key Decisions
- **Binary location**: `src/bin/merge-dirs-for-linux-limit.rs` (standard Cargo bin layout)
- **Argument structure**: `<src> <dst> --dry-run` (positional args + flag)
- **Output format**: Simple text - `{src_path} -> {dst_path}`
- **Path handling**: Use `canonicalize()` for reliable `strip_prefix`
- **Validation**: Check src exists and is directory before traversal

### Deferred to Later Phases
- Symlink detection/skipping (Phase 3)
- `new_filename()` integration (Phase 3) - has `fs::create_dir_all` side effect
- JSON/structured output format
- Progress reporting
- Detailed error types

## STEPs

- [ ] STEP 1: Create CLI binary with argument parsing and validation [dry-coded]
- [ ] STEP 2: Implement directory traversal with relative path computation and output [dry-coded]

See `STEP-1-cli-args.md` and `STEP-2-traversal-output.md` for detailed implementation plans.

## Review Status

- [x] Initial review completed
- [x] Review feedback applied (assert -> bail!, dst path clarification)
- [x] Final verification completed
