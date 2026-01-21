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
- **MUST use streaming hash** - archives may include large files (50GB+)

## Research Synthesis

### Accepted (from multi-agent research)
- **Streaming hash**: Essential for handling large archive files (Gemini/Codex agreed)
- **Function location**: lib.rs (all agreed)
- **Size-first comparison**: Standard practice (all agreed)
- **sha2 crate**: Version 0.10, use `io::copy` or read loop pattern

### Deferred (all agreed)
- Hash caching (single pass tool)
- Parallel hashing (Phase 4 if needed)
- mmap (unnecessary complexity)
- Complex conflict resolution (Phase 3)

## STEPs

- [ ] STEP 1: Add sha2 dependency and implement streaming comparison functions in lib.rs
- [ ] STEP 2: Integrate comparison into CLI and output results with status labels
- [ ] Initial review with multiple agents
- [ ] Apply review feedback (iteration 1)
- [ ] Second review with multiple agents
- [ ] Apply review feedback if needed (iteration 2)
- [ ] Final validation

## Technical Details

### Dependencies to Add
```toml
sha2 = "0.10"
```

### Function Signatures (lib.rs)
```rust
pub fn files_have_same_size(src: &Path, dst: &Path) -> io::Result<bool>
pub fn files_have_same_content(src: &Path, dst: &Path) -> io::Result<bool>
```

### Streaming Hash Pattern
```rust
use sha2::{Sha256, Digest};
use std::io::{self, BufReader, Read};

fn compute_sha256(path: &Path) -> io::Result<[u8; 32]> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize().into())
}
```

### CLI Output Format
```
[new] /src/path/file.txt -> /dst/path/file.txt
[duplicate] /src/path/same.txt -> /dst/path/same.txt
[conflict (size)] /src/path/diff.txt -> /dst/path/diff.txt
[conflict (content)] /src/path/diff2.txt -> /dst/path/diff2.txt
```

## Status

**Current**: Stage 4 - IMPROVE PLAN complete, proceeding to EXECUTE (Dry Coding)
