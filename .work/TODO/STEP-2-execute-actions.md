# STEP 2: Execute Actions

## Objective

Implement `execute_action()` that performs the actual file operations based on the `Action` enum from STEP 1. Handle both dry-run (preview) and actual execution modes.

## Design

### 1. execute_action Function Signature

```rust
/// Executes the resolved action for a file.
///
/// # Arguments
/// * `action` - The action to perform
/// * `src` - Absolute path to source file
/// * `dry_run` - If true, only print what would be done
///
/// # Returns
/// Result indicating success or failure
pub fn execute_action(action: &Action, src: &Path, dry_run: bool) -> Result<()> {
    match action {
        Action::Move { target } => execute_move(src, target, dry_run),
        Action::DeleteSrcOnly { target } => execute_delete_src(src, target, dry_run),
        Action::Skip => Ok(()),
        Action::Error { message } => {
            log::error!("[error] {}: {}", src.display(), message);
            Ok(())  // Continue processing other files
        }
    }
}
```

### 2. Output Format

Consistent format for all operations:

```
[action] src -> target
```

Examples:
```
[move] /src/dir/file.txt -> /dst/dir/file.txt
[move] /src/dir/longname.txt -> /dst/dir/longna.txt  (shortened)
[move] /src/dir/conflict.txt -> /dst/dir/conflict.1.txt  (suffix)
[delete-src] /src/dir/dup.txt -> /dst/dir/dup.txt  (duplicate)
[error] /src/dir/bad.txt: Permission denied
[skip] /src/dir/skip.txt
```

### 3. Move Operation

```rust
fn execute_move(src: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[move] {} -> {}", src.display(), target.display());
        return Ok(());
    }

    // Create parent directories (may already exist from resolve_action)
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // Use jdt::rename_file for cross-filesystem support (handles EXDEV)
    jdt::rename_file(src, target)?;

    // Verify target exists after move
    if !target.exists() {
        anyhow::bail!("Move verification failed: target not found after move");
    }

    println!("[move] {} -> {}", src.display(), target.display());
    Ok(())
}
```

### 4. Delete Source Only Operation

```rust
fn execute_delete_src(src: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[delete-src] {} -> {} (duplicate)", src.display(), target.display());
        return Ok(());
    }

    // Delete source file (target already has identical content)
    fs::remove_file(src)?;

    println!("[delete-src] {} -> {} (duplicate)", src.display(), target.display());
    Ok(())
}
```

### 5. Integration in Binary

```rust
// In main() after getting action from resolve_action:

jdt::walk_dir(&src, |file_path| {
    let rel_path = file_path.strip_prefix(&src).expect("file must be under src");

    let action = resolve_action(&file_path, &dst, &rel_path);

    if let Err(e) = execute_action(&action, &file_path, args.dry_run) {
        log::error!("Failed to execute action for {}: {}", file_path.display(), e);
        // Continue with next file (fail-soft)
    }
});
```

## Key Design Decisions

### Why separate resolve_action and execute_action?

1. **Testability**: Can unit test resolution logic without filesystem side effects
2. **Dry-run simplicity**: Resolution happens identically; only execution differs
3. **Logging clarity**: Action is known before execution, so output is predictable

### Why log after operation (not before)?

For actual execution, logging after confirms the operation completed. For dry-run, we log immediately (nothing to confirm).

### Error handling strategy

- Log errors but continue processing other files (fail-soft)
- `Action::Error` from resolution is logged but doesn't abort
- `execute_action` errors are logged at caller level
- Comprehensive error handling deferred to Phase 4

## Impact Scope

| File | Change |
|------|--------|
| `src/lib.rs` | Add `execute_action()`, `execute_move()`, `execute_delete_src()` |
| `src/bin/merge-dirs-for-linux-limit.rs` | Replace status printing with `resolve_action` + `execute_action` flow |

## Dependencies

- STEP 1 must be completed first (provides `Action` enum and `resolve_action`)

## Verification Method

### 1. Unit tests

```rust
#[test]
fn test_execute_move_dry_run() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let target = dir.path().join("target.txt");
    fs::write(&src, "hello").unwrap();

    let action = Action::Move { target: target.clone() };
    execute_action(&action, &src, true).unwrap();

    // Source still exists (dry-run)
    assert!(src.exists());
    // Target not created (dry-run)
    assert!(!target.exists());
}

#[test]
fn test_execute_move_actual() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let target = dir.path().join("target.txt");
    fs::write(&src, "hello").unwrap();

    let action = Action::Move { target: target.clone() };
    execute_action(&action, &src, false).unwrap();

    // Source removed
    assert!(!src.exists());
    // Target created with correct content
    assert!(target.exists());
    assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
}

#[test]
fn test_execute_delete_src() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let target = dir.path().join("target.txt");
    fs::write(&src, "hello").unwrap();
    fs::write(&target, "hello").unwrap();

    let action = Action::DeleteSrcOnly { target: target.clone() };
    execute_action(&action, &src, false).unwrap();

    // Source removed
    assert!(!src.exists());
    // Target unchanged
    assert!(target.exists());
}
```

### 2. Build verification

```bash
cargo build
cargo test
cargo clippy
```

### 3. Integration test (manual)

```bash
# Setup
mkdir -p /tmp/merge-test-src/subdir
mkdir -p /tmp/merge-test-dst
echo "new file" > /tmp/merge-test-src/new.txt
echo "duplicate" > /tmp/merge-test-src/dup.txt
echo "duplicate" > /tmp/merge-test-dst/dup.txt
echo "conflict src" > /tmp/merge-test-src/conflict.txt
echo "conflict dst" > /tmp/merge-test-dst/conflict.txt
echo "nested" > /tmp/merge-test-src/subdir/nested.txt

# Dry run
cargo run --bin merge-dirs-for-linux-limit -- /tmp/merge-test-src /tmp/merge-test-dst --dry-run

# Expected output:
# [move] /tmp/merge-test-src/new.txt -> /tmp/merge-test-dst/new.txt
# [delete-src] /tmp/merge-test-src/dup.txt -> /tmp/merge-test-dst/dup.txt (duplicate)
# [move] /tmp/merge-test-src/conflict.txt -> /tmp/merge-test-dst/conflict.1.txt
# [move] /tmp/merge-test-src/subdir/nested.txt -> /tmp/merge-test-dst/subdir/nested.txt

# Verify nothing changed
ls /tmp/merge-test-src/  # All files still present

# Actual run
cargo run --bin merge-dirs-for-linux-limit -- /tmp/merge-test-src /tmp/merge-test-dst

# Verify
ls /tmp/merge-test-src/  # Should be empty (or just subdir if we don't clean empty dirs)
ls /tmp/merge-test-dst/  # Should have: new.txt, dup.txt, conflict.txt, conflict.1.txt, subdir/nested.txt
```

## Notes

- `jdt::rename_file` handles EXDEV (cross-filesystem) by doing copy+delete internally
- Verification step (target.exists() after move) is a safety check for critical archives
- Empty source directory cleanup is out of scope (Sprint constraint)
- Summary statistics deferred to Phase 4
