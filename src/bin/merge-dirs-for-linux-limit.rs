use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "merge-dirs-for-linux-limit")]
#[command(about = "Preview directory merge operations for Linux filename limits")]
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

    // Traverse source directory and print planned operations
    jdt::walk_dir(&src, |file_path| {
        // Compute relative path from source root
        let rel_path = file_path.strip_prefix(&src).expect("file must be under src");

        // Compute target path
        let target_path = dst.join(rel_path);

        // Output planned operation
        println!("{} -> {}", file_path.display(), target_path.display());
    });

    Ok(())
}
