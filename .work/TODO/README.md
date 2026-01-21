# Phase 4: Error Handling + Polish + Junk File Filtering

## Core Context

**PROBLEM**: The merge tool lacks user-friendly output and handles errors silently. Additionally, OS-specific junk files (macOS `.DS_Store`, etc.) cause permission errors and should be skipped.

**SUCCESS**:
- Permission errors are logged but don't abort entire operation
- Clear summary at end: X moved, Y skipped (junk), Z duplicates, W errors
- Dry-run output accurately predicts actual behavior
- OS junk files (macOS, Windows, Linux) are automatically skipped
- All operations are logged at appropriate verbosity levels

**NOT DOING**:
- Configurable junk file patterns (hardcoded list only)
- Verbose/quiet CLI flags (use RUST_LOG for now)
- Progress bar or ETA display
- Interactive confirmation prompts

**TRIGGER**:
> 古い Mac を FUSE ベースの SFTP クライアント（Gnome の Files）でマウントして cp とか mv とかしたいときになんとか雑に名前を短くしたいので作った記憶がある
> /Volumes/Data -> /data のように２つの directory を deep に merge しつつ、名前が長かったら短くしつつ、データが同じかどうかを file size + CRC とかで確認しつつファイルを mac から linux に移したい
> 非常に大量かつ重要なアーカイブ的なものもありそうなので dry-run は必須

**Phase 4 Addendum**:
> 実際の mac からの import となると以下のファイルを無視するようにしたい。権限周りでエラーになるので、これらを無視したい。ただ mac 専用のツールに見せたくないので windows とか linux とかの junk ファイルは skip するような仕様を入れつつ。
> - `.DocumentRevisions-V100`, `.DS_Store`, `.fseventsd`, `.Spotlight-V100`, `.TemporaryItems`, `.Trashes`

**PURPOSE**: Complete the tool with production-ready error handling, clear feedback, and automatic filtering of OS junk files.

**CONSTRAINTS**:
- Reuse existing `Action::Skip` for junk files
- Keep junk file list hardcoded (no config file)
- Summary output to stdout, errors to stderr
- Maintain fail-soft approach (continue on errors)

## Phase Overview

This phase adds two main capabilities:
1. **Junk File Filtering**: Automatically skip OS-specific junk files
2. **Summary Statistics**: Provide clear operation counts at completion

## STEPs

- [ ] STEP 1: Add junk path detection and skip logic
  - Add `is_junk_path(path: &Path) -> bool` in lib.rs
  - Check path components (not just filename) to skip files under junk directories
  - Call at start of `resolve_action()`, return `Action::Skip` for junk
  - Case sensitivity: macOS exact match, Windows case-insensitive

- [ ] STEP 2: Add summary statistics collection and output
  - Track counts: moved, duplicates, skipped (combined), errors
  - Modify main loop to collect stats from action results
  - Print summary at end (both dry-run and actual execution)

- [ ] STEP 3: Verify dry-run consistency (may be N/A)
  - Confirm dry-run output matches actual execution behavior
  - Ensure summary is printed in both modes

## Technical Notes

### Junk Files to Skip

**macOS** (exact match, case-sensitive):
- `.DS_Store` - Finder metadata
- `.DocumentRevisions-V100` - Document versioning
- `.fseventsd` - File system events
- `.Spotlight-V100` - Spotlight index
- `.TemporaryItems` - Temp files
- `.Trashes` - Trash folder
- `.AppleDouble` - Resource forks
- `.AppleDB` - Apple database
- `.AppleDesktop` - Desktop database
- `.VolumeIcon.icns` - Volume icon
- `.com.apple.timemachine.donotpresent` - Time Machine marker
- `.apdisk` - Network share marker
- `._*` - AppleDouble files (prefix pattern, `starts_with`)

**Windows** (case-insensitive):
- `desktop.ini` - Folder settings
- `Thumbs.db` - Thumbnail cache
- `ehthumbs.db` - Video thumbnails
- `$RECYCLE.BIN` - Recycle bin

**Linux** (exact match):
- `lost+found` - fsck recovery directory

### Research Findings Summary

**Accepted**:
- Check path components (not just filename) to catch files under junk directories
- macOS names: case-sensitive exact match
- Windows names: case-insensitive match
- Place junk check at start of `resolve_action()` for consistency with symlink skip

**Rejected**:
- Replace `jdt::walk_dir` with `walkdir` crate (Sprint constraint to use jdt)
- Root-only scope for volume dirs (overcomplicates, low false positive risk for use case)
- New `Action::Skip { reason }` variant (track stats in main loop instead)
- Configurable patterns (YAGNI)

### Implementation Approach

1. Add `is_junk_path(path: &Path) -> bool` in lib.rs
   - Check each path component against junk list
   - Early return true if any component matches
2. Call at start of `resolve_action()` before symlink check
3. Reuse existing `Action::Skip` (no new variant)
4. In main loop: track action counts, print summary at end
5. Summary format: `Moved: X, Duplicates: Y, Skipped: Z, Errors: W`

### Cross-Device Support
Confirmed: `jdt::rename_file` handles EXDEV (cross-filesystem) via copy+delete internally. No changes needed.
