use anyhow::Result;
use clap::Parser;
use rename_for_linux_limit::{execute_action, resolve_action};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "merge-dirs-for-linux-limit")]
#[command(about = "Merge directories with automatic filename shortening for Linux limits")]
struct Args {
    /// Source directory to merge from
    src: PathBuf,

    /// Destination directory to merge into
    dst: PathBuf,

    /// Show planned operations without executing
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    // Validate src exists and is directory
    let src = args.src.canonicalize()?;
    if !src.is_dir() {
        anyhow::bail!("Source must be a directory: {}", src.display());
    }

    // Canonicalize dst only if it exists, otherwise use as-is
    let dst = if args.dst.exists() {
        args.dst.canonicalize()?
    } else {
        args.dst
    };

    // Traverse source directory and execute operations
    jdt::walk_dir(&src, |file_path| {
        // Compute relative path from source root
        let rel_path = file_path
            .strip_prefix(&src)
            .expect("file must be under src");

        // Resolve what action to take
        let action = resolve_action(&file_path, &dst, rel_path);

        // Execute the action
        if let Err(e) = execute_action(&action, &file_path, args.dry_run) {
            log::error!(
                "Failed to execute action for {}: {}",
                file_path.display(),
                e
            );
            // Continue with next file (fail-soft)
        }
    });

    Ok(())
}
