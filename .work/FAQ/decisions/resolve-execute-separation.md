# Separate resolve_action() from execute_action()

## Decision

Separate `resolve_action()` (pure, read-only) from `execute_action()` (side effects) with an `Action` enum bridging them.

## Context

The `merge-dirs-for-linux-limit` tool needs `--dry-run` support that accurately reflects what real execution would do. The tool also needs to handle complex conflict resolution scenarios:

- Rename files that exceed Linux filename length limits
- Skip files that already exist at the destination with identical content
- Delete source files when destination already has them (to avoid re-processing)

Getting dry-run accuracy wrong means users cannot trust the preview. Getting conflict resolution wrong means data loss or corruption.

## Options Considered

1. **Single function with conditional execution**: One function checks conditions and optionally performs the action based on a `dry_run` flag passed in
2. **Separate pure resolution and effectful execution**: A pure `resolve_action()` determines what to do, returns an `Action` enum, and `execute_action()` performs the side effects

## Rationale

Option 2 was chosen because it provides:

- **Clean testability**: `resolve_action()` can be unit tested without filesystem mocking - just pass paths and assert the returned Action
- **Guaranteed dry-run accuracy**: Resolution logic is identical for both dry-run and real execution because it is the same code path
- **Clear separation**: Prevents accidental mutations during planning phase - the type system enforces that `resolve_action()` cannot have side effects
- **Explicit outcomes**: The `Action` enum makes all possible outcomes explicit and type-safe, preventing forgotten cases

## Consequences

**Positive**:
- Clean architecture with clear boundaries
- Easy unit testing of resolution logic
- Reliable dry-run that users can trust
- Self-documenting code via the Action enum

**Negative**:
- Action enum adds some boilerplate
- Two functions to maintain instead of one

## Date

2026-01-20 (Sprint: merge-dirs-for-linux-limit, Phase 3)
