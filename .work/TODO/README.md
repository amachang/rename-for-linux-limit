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

- [ ] STEP 1: Add junk file detection and skip logic
- [ ] STEP 2: Add summary statistics collection and output
- [ ] STEP 3: Review and verify dry-run consistency

## Technical Notes

### Junk Files to Skip

**macOS**:
- `.DS_Store` - Finder metadata
- `.DocumentRevisions-V100` - Document versioning
- `.fseventsd` - File system events
- `.Spotlight-V100` - Spotlight index
- `.TemporaryItems` - Temp files
- `.Trashes` - Trash folder
- `.AppleDouble` - Resource forks
- `.AppleDB` - Apple database
- `.AppleDesktop` - Desktop database
- `._*` - AppleDouble files (resource fork prefix)

**Windows**:
- `desktop.ini` - Folder settings
- `Thumbs.db` - Thumbnail cache
- `ehthumbs.db` - Video thumbnails
- `$RECYCLE.BIN` - Recycle bin

**Linux**:
- `lost+found` - fsck recovery directory

### Implementation Approach

1. Add `is_junk_file()` function in lib.rs
2. Call before `resolve_action()` or integrate into it
3. Use `Action::Skip` with reason for junk files (or new variant)
4. Collect statistics during walk, print summary at end

### Cross-Device Support
Confirmed: `jdt::rename_file` handles EXDEV (cross-filesystem) via copy+delete internally. No changes needed.
