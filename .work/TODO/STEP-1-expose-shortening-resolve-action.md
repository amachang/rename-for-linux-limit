# STEP 1: Expose Shortening Logic + Resolve Action

## Objective

Create the `Action` enum and `resolve_action()` function that determines what operation to perform for each source file. This replaces the current string-based `determine_file_status()`.

## Design

### 1. Make `new_candidate_filename` callable

The existing `new_candidate_filename` function at lib.rs:107-258 is private. Two options:

**Option A: Make it `pub`** (Chosen)
- Simplest change: just add `pub` keyword
- Exposes implementation detail but acceptable for internal use

**Option B: Create wrapper function**
- More indirection, no real benefit for this crate

Decision: **Option A** - just make `new_candidate_filename` public.

### 2. Action Enum (lib.rs)

```rust
/// Action to perform for a source file during merge
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Move src to target (target path includes shortened filename)
    Move { target: PathBuf },
    /// Delete src only - target already has identical content
    DeleteSrcOnly { target: PathBuf },
    /// Skip this file (should not happen in normal flow)
    Skip,
    /// Error occurred during resolution
    Error { message: String },
}
```

### 3. resolve_action Function (lib.rs)

```rust
/// Resolves what action to take for a source file being merged to destination.
///
/// # Arguments
/// * `src` - Absolute path to source file
/// * `dst_dir` - Absolute path to destination directory root
/// * `rel_path` - Relative path from src root (preserves directory structure)
///
/// # Returns
/// Action enum indicating what operation to perform
pub fn resolve_action(src: &Path, dst_dir: &Path, rel_path: &Path) -> Action {
    // Get parent directory and filename from rel_path
    let rel_parent = rel_path.parent().unwrap_or(Path::new(""));
    let filename = match rel_path.file_name() {
        Some(f) => f.to_string_lossy().to_string(),
        None => return Action::Error { message: "No filename in path".to_string() },
    };

    // Build target directory
    let target_dir = dst_dir.join(rel_parent);

    // Create target directory (OK even in dry-run - only files matter)
    if let Err(e) = fs::create_dir_all(&target_dir) {
        return Action::Error { message: format!("Cannot create directory: {}", e) };
    }

    // Load config once per invocation (consider caching at caller level)
    let config = jdt::project(crate_name!()).config::<Config>();
    let ignored_tags: HashSet<String> = config.ignored_tags.iter().map(|s| normalize_str(s)).collect();
    let tag_conversion_map: HashMap<String, String> = config.conversions.iter()
        .map(|(k, v)| (normalize_str(k), normalize_str(v)))
        .collect();

    // Loop with increasing n_retries until resolution found
    let mut n_retries = 0;
    loop {
        let candidate = new_candidate_filename(&filename, &ignored_tags, &tag_conversion_map, n_retries);
        let target = target_dir.join(&candidate);

        if !target.exists() {
            // Target doesn't exist - move is safe
            return Action::Move { target };
        }

        // Target exists - check if duplicate
        match files_have_same_size(src, &target) {
            Ok(true) => {
                // Same size - compare content
                match files_have_same_content(src, &target) {
                    Ok(true) => {
                        // Identical content - delete src only
                        return Action::DeleteSrcOnly { target };
                    }
                    Ok(false) => {
                        // Different content - try next suffix
                        n_retries += 1;
                        continue;
                    }
                    Err(e) => {
                        return Action::Error { message: format!("Hash error: {}", e) };
                    }
                }
            }
            Ok(false) => {
                // Different size - conflict, try next suffix
                n_retries += 1;
                continue;
            }
            Err(e) => {
                return Action::Error { message: format!("Size check error: {}", e) };
            }
        }
    }
}
```

### 4. Config Loading Optimization

The config loading inside the loop is inefficient. Consider:

**Option A: Load once per file** (current design)
- Simple, correct
- Config is cached by jdt internally anyway

**Option B: Pass config as parameter**
- Caller loads once, passes to all resolve_action calls
- More efficient but adds complexity

Decision: **Option A** for now. jdt caches config, so the overhead is minimal. Can optimize in Phase 4 if profiling shows it's a bottleneck.

## Key Code Snippets

### Making new_candidate_filename public (lib.rs line 107)

```rust
// Before
fn new_candidate_filename(

// After
pub fn new_candidate_filename(
```

### Imports needed in lib.rs

```rust
// Already present:
use std::path::PathBuf;
use std::fs;
// Need to ensure these are used in resolve_action
```

## Impact Scope

| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub` to `new_candidate_filename`, add `Action` enum, add `resolve_action` function |
| `src/bin/merge-dirs-for-linux-limit.rs` | Replace `determine_file_status()` with `resolve_action()` call |

## Integration with Existing Code

### Current flow (Phase 2):
```
walk_dir -> determine_file_status(src, dst) -> print status string
```

### New flow (Phase 3):
```
walk_dir -> resolve_action(src, dst_dir, rel_path) -> Action enum -> execute_action (STEP 2)
```

The `determine_file_status` function in the binary will be deleted - its logic is absorbed into `resolve_action`.

## Verification Method

1. **Unit tests for resolve_action**:
   - `new` file (target doesn't exist) -> `Action::Move`
   - `duplicate` file (target exists, same content) -> `Action::DeleteSrcOnly`
   - `conflict` file (target exists, different content) -> `Action::Move` with suffix
   - Long filename -> `Action::Move` with shortened name

2. **Build verification**:
   ```bash
   cargo build
   cargo test
   cargo clippy
   ```

3. **Manual smoke test**:
   ```bash
   # Create test dirs
   mkdir -p /tmp/test-src /tmp/test-dst
   echo "hello" > /tmp/test-src/test.txt

   # Run (after STEP 2 implements execute_action)
   cargo run --bin merge-dirs-for-linux-limit -- /tmp/test-src /tmp/test-dst --dry-run
   ```

## Notes

- The `Skip` action variant is kept for completeness but shouldn't occur in normal flow
- Error handling is minimal (log and continue) - comprehensive handling deferred to Phase 4
- Config caching optimization deferred - jdt caches internally
