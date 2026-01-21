# STEP 3: Verify Dry-Run Consistency

## Objective

Confirm that dry-run output accurately predicts actual execution behavior, and that summary statistics are printed in both modes.

## Analysis of Current Behavior

### Current Implementation Review

Looking at `execute_action` in `lib.rs` (lines 508-523):

```rust
pub fn execute_action(action: &Action, src: &Path, dry_run: bool) -> Result<()> {
    match action {
        Action::Move { target } => execute_move(src, target, dry_run),
        Action::DeleteSrcOnly { target } => execute_delete_src(src, target, dry_run),
        Action::Skip => {
            if dry_run {
                println!("[skip] {}", src.display());
            }
            Ok(())
        }
        Action::Error { message } => {
            eprintln!("[error] {}: {}", src.display(), message);
            Ok(()) // Continue processing other files
        }
    }
}
```

And `execute_move` (lines 525-541):

```rust
fn execute_move(src: &Path, target: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("[move] {} -> {}", src.display(), target.display());
        return Ok(());
    }
    // ... actual execution prints same message after move
    println!("[move] {} -> {}", src.display(), target.display());
    Ok(())
}
```

### Consistency Check

| Action Type | Dry-run Output | Actual Output | Consistent? |
|-------------|----------------|---------------|-------------|
| Move | `[move] src -> dst` | `[move] src -> dst` | Yes |
| DeleteSrcOnly | `[delete-src] src -> dst (duplicate)` | `[delete-src] src -> dst (duplicate)` | Yes |
| Skip | `[skip] src` | (nothing) | **Inconsistent** |
| Error | `[error] src: msg` | `[error] src: msg` | Yes |

### Issue Found: Skip Behavior

In dry-run mode, `Action::Skip` prints `[skip] src`, but in actual execution mode, it prints nothing. This is an **intentional design choice** in the current code - skip means "do nothing", so printing nothing makes sense.

However, for user feedback consistency, we should decide if this is desired:

**Option A: Keep current behavior (recommended)**
- Dry-run shows all planned operations including skips for user review
- Actual run only shows actions performed (moves, deletes)
- Rationale: Skips are "non-events" in actual execution

**Option B: Print skips in both modes**
- Consistent output between modes
- May be verbose for directories with many junk files

**Recommendation**: Keep Option A. The dry-run purpose is preview, so showing skips is valuable. During actual execution, silence for skips is appropriate.

## Implementation Details

### No Code Changes Required

The current implementation is already consistent in a meaningful way:
- All action types are resolved identically regardless of `dry_run` flag
- The same `Action` enum value produces the same stat category
- Output format matches (same prefixes, same path display)

### Summary Statistics (from STEP 2)

With STEP 2 implemented, both modes will print summary because stats are collected from action resolution (before `execute_action`):

```rust
// This happens in both dry-run and actual mode
match &action {
    Action::Move { .. } => stats.moved += 1,
    // ...
}

// Summary printed at end regardless of dry_run
print_summary(&stats, args.dry_run);
```

## Verification Method

### Test Script

```bash
#!/bin/bash
set -e

# Setup
TEST_DIR=$(mktemp -d)
SRC="$TEST_DIR/src"
DST="$TEST_DIR/dst"

mkdir -p "$SRC/subdir"
echo "content1" > "$SRC/file1.txt"
echo "content2" > "$SRC/subdir/file2.txt"
touch "$SRC/.DS_Store"

mkdir -p "$DST"
echo "content1" > "$DST/file1.txt"  # Pre-existing duplicate

echo "=== Dry-run ==="
cargo run -- "$SRC" "$DST" --dry-run

echo ""
echo "=== Actual run ==="
# Reset test
rm -rf "$DST"
mkdir -p "$DST"
echo "content1" > "$DST/file1.txt"

cargo run -- "$SRC" "$DST"

echo ""
echo "=== Verify results ==="
ls -la "$DST"
ls -la "$DST/subdir" 2>/dev/null || echo "subdir not created (expected if only skip/dup)"

# Cleanup
rm -rf "$TEST_DIR"
```

### Expected Output Comparison

**Dry-run:**
```
[skip] /tmp/.../src/.DS_Store
[delete-src] /tmp/.../src/file1.txt -> /tmp/.../dst/file1.txt (duplicate)
[move] /tmp/.../src/subdir/file2.txt -> /tmp/.../dst/subdir/file2.txt

[dry-run] Summary: 1 moved, 1 duplicates, 1 skipped, 0 errors (3 total)
```

**Actual run:**
```
[delete-src] /tmp/.../src/file1.txt -> /tmp/.../dst/file1.txt (duplicate)
[move] /tmp/.../src/subdir/file2.txt -> /tmp/.../dst/subdir/file2.txt

Summary: 1 moved, 1 duplicates, 1 skipped, 0 errors (3 total)
```

Note: Skip line absent in actual run (intentional), but skip count still appears in summary.

## Impact Scope

| File | Change Type |
|------|-------------|
| None | No code changes - verification only |

## Design Notes

- **Why different skip output is acceptable**: Dry-run is for preview/planning. Showing "will skip" is useful. During actual execution, skipping is a non-event that doesn't need logging (users care about what DID happen).

- **Summary is the equalizer**: Even though individual skip lines differ, the summary provides consistent counts in both modes. This is the key consistency that matters for user trust.

- **STEP 3 may be marked N/A**: If review confirms current behavior is acceptable, this STEP produces documentation but no code changes. This is valid - verification steps exist to catch issues, not guarantee code changes.
