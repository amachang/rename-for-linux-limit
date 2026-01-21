# STEP 1: Junk Path Detection

## Objective

Add automatic detection and skipping of OS-specific junk files (macOS, Windows, Linux) to prevent permission errors and avoid processing unnecessary metadata files.

## Implementation Details

### 1. Add Junk Path Constants (lib.rs, after line 30)

Add static arrays for junk patterns, grouped by OS:

```rust
/// macOS junk files (exact match, case-sensitive)
const MACOS_JUNK_EXACT: &[&str] = &[
    ".DS_Store",
    ".DocumentRevisions-V100",
    ".fseventsd",
    ".Spotlight-V100",
    ".TemporaryItems",
    ".Trashes",
    ".AppleDouble",
    ".AppleDB",
    ".AppleDesktop",
    ".VolumeIcon.icns",
    ".com.apple.timemachine.donotpresent",
    ".apdisk",
    "._", // AppleDouble stub (edge case: exact "._" without filename)
];

/// macOS junk prefix (AppleDouble resource forks)
const MACOS_JUNK_PREFIX: &str = "._";

/// Windows junk files (case-insensitive)
const WINDOWS_JUNK: &[&str] = &[
    "desktop.ini",
    "thumbs.db",
    "ehthumbs.db",
    "$recycle.bin",
];

/// Linux junk files (exact match)
const LINUX_JUNK: &[&str] = &[
    "lost+found",
];
```

### 2. Add `is_junk_path` Function (lib.rs, after constants)

```rust
/// Checks if any component of the path matches known junk file patterns.
///
/// Checks all path components, not just the filename, to catch files
/// nested under junk directories (e.g., `.Trashes/501/file.txt`).
pub fn is_junk_path(path: &Path) -> bool {
    for component in path.components() {
        let name = match component {
            std::path::Component::Normal(s) => s.to_string_lossy(),
            _ => continue,
        };

        // macOS: exact match (case-sensitive)
        if MACOS_JUNK_EXACT.contains(&name.as_ref()) {
            return true;
        }

        // macOS: prefix match for AppleDouble files
        if name.starts_with(MACOS_JUNK_PREFIX) && name.len() > MACOS_JUNK_PREFIX.len() {
            return true;
        }

        // Windows: case-insensitive match
        let name_lower = name.to_ascii_lowercase();
        if WINDOWS_JUNK.contains(&name_lower.as_str()) {
            return true;
        }

        // Linux: exact match
        if LINUX_JUNK.contains(&name.as_ref()) {
            return true;
        }
    }

    false
}
```

### 3. Modify `resolve_action` (lib.rs, line 393)

Insert junk check at the very start of `resolve_action`, before the symlink check:

```rust
pub fn resolve_action(src: &Path, dst_dir: &Path, rel_path: &Path) -> Action {
    // Skip junk files (OS metadata that causes permission errors)
    if is_junk_path(rel_path) {
        return Action::Skip;
    }

    // Skip symlinks (Sprint scope exclusion)
    if src
        .symlink_metadata()
        // ... existing code continues
```

**Why `rel_path` not `src`**: The relative path preserves the directory structure from source root. This correctly identifies junk like `subdir/.DS_Store` while avoiding false positives from absolute path components like `/home/user/.config/`.

## Code Snippets

### Complete `is_junk_path` with edge case handling

```rust
pub fn is_junk_path(path: &Path) -> bool {
    for component in path.components() {
        let name = match component {
            std::path::Component::Normal(s) => s.to_string_lossy(),
            _ => continue, // Skip RootDir, CurDir, ParentDir, Prefix
        };

        // macOS: exact match (case-sensitive)
        if MACOS_JUNK_EXACT.contains(&name.as_ref()) {
            return true;
        }

        // macOS: prefix match for AppleDouble files (._filename)
        // Require length > prefix to avoid matching just "._"
        if name.starts_with(MACOS_JUNK_PREFIX) && name.len() > MACOS_JUNK_PREFIX.len() {
            return true;
        }

        // Windows: case-insensitive match
        let name_lower = name.to_ascii_lowercase();
        if WINDOWS_JUNK.contains(&name_lower.as_str()) {
            return true;
        }

        // Linux: exact match
        if LINUX_JUNK.contains(&name.as_ref()) {
            return true;
        }
    }

    false
}
```

## Impact Scope

| File | Function/Area | Change Type |
|------|---------------|-------------|
| `src/lib.rs` | Module constants | Add 4 const arrays |
| `src/lib.rs` | `is_junk_path()` | New function |
| `src/lib.rs` | `resolve_action()` | Add 3-line check at start |

## Verification Method

### Unit Tests to Add

```rust
#[cfg(test)]
mod junk_tests {
    use super::*;

    #[test]
    fn test_macos_junk_exact() {
        assert!(is_junk_path(Path::new(".DS_Store")));
        assert!(is_junk_path(Path::new(".Trashes")));
        assert!(is_junk_path(Path::new("subdir/.DS_Store")));
        assert!(is_junk_path(Path::new(".Spotlight-V100/deeply/nested/file.txt")));
    }

    #[test]
    fn test_macos_junk_prefix() {
        assert!(is_junk_path(Path::new("._photo.jpg")));
        assert!(is_junk_path(Path::new("subdir/._document.pdf")));
        // Edge case: just "._" is also junk (AppleDouble stub)
        assert!(is_junk_path(Path::new("._")));
    }

    #[test]
    fn test_windows_junk_case_insensitive() {
        assert!(is_junk_path(Path::new("desktop.ini")));
        assert!(is_junk_path(Path::new("DESKTOP.INI")));
        assert!(is_junk_path(Path::new("Desktop.Ini")));
        assert!(is_junk_path(Path::new("Thumbs.db")));
        assert!(is_junk_path(Path::new("$RECYCLE.BIN")));
        assert!(is_junk_path(Path::new("$Recycle.Bin/file.txt")));
    }

    #[test]
    fn test_linux_junk() {
        assert!(is_junk_path(Path::new("lost+found")));
        assert!(is_junk_path(Path::new("lost+found/recovered_file")));
    }

    #[test]
    fn test_not_junk() {
        assert!(!is_junk_path(Path::new("normal_file.txt")));
        assert!(!is_junk_path(Path::new("subdir/file.jpg")));
        assert!(!is_junk_path(Path::new(".hidden_but_not_junk")));
        assert!(!is_junk_path(Path::new("DS_Store"))); // Missing leading dot
    }
}
```

### Integration Test

```bash
# Create test structure with junk files
mkdir -p /tmp/test_src/.Spotlight-V100/nested
touch /tmp/test_src/.DS_Store
touch /tmp/test_src/normal.txt
touch /tmp/test_src/.Spotlight-V100/nested/file.txt

# Run dry-run
cargo run -- /tmp/test_src /tmp/test_dst --dry-run

# Expected output:
# [skip] /tmp/test_src/.DS_Store
# [skip] /tmp/test_src/.Spotlight-V100/nested/file.txt
# [move] /tmp/test_src/normal.txt -> /tmp/test_dst/normal.txt
```

## Design Notes

- **Why check `rel_path` not `src`**: Using the relative path avoids false positives from absolute path components. For example, if the source is `/Users/john/.Trashes_backup/important.txt`, we should NOT skip it just because the absolute path contains something similar to `.Trashes`.

- **Why early return pattern**: The function iterates components and returns `true` on first match. This is efficient and follows existing patterns in the codebase (see symlink check).

- **Case sensitivity rationale**: macOS HFS+ is case-insensitive but case-preserving. However, the junk files are created by the OS with exact casing, so exact match is sufficient. Windows NTFS is case-insensitive, requiring lowercase comparison.
