# rename-for-linux-limit

A Rust utility for renaming files to comply with Linux filename length limits.

## Development Lifecycle

### Commands

| Action | Command |
|--------|---------|
| Build | `cargo build --release` |
| Test | `cargo test` |
| Lint | `cargo clippy` |
| Format | `cargo fmt` |

### Project Type
- **Language**: Rust (edition 2021)
- **Package Manager**: Cargo
- **Build Tool**: Cargo

### Key Paths
- **Source**: `src/`
- **Build output**: `target/`
- **Tests**: `src/lib.rs` (`#[cfg(test)]` module)

### Local Setup
```bash
cargo build
```

### Notes
- Private dependency `jdt` requires SSH access to `git@github.com/amachang/jdt.git`
