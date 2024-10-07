use std::{path::PathBuf, fs, io};
use clap::Parser;
use anyhow::Result;

use rename_for_linux_limit::new_filename;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short = 's', long, default_value = "false")]
    only_show_new_filename: bool,
    #[clap(short = 'd', long, help = "If not set --dst-dir, the same as the given path's parent dir.")]
    dst_dir: Option<PathBuf>,
    path: PathBuf,
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Rename error: {0} -> {1}: {2}")]
    RenameError(PathBuf, PathBuf, io::Error),
    #[error("Filename not found in path: {0}")]
    FilenameNotFound(PathBuf),
    #[error("Unknown error: {0}")]
    UnknownError(#[from] anyhow::Error),
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let path = args.path;
    let dst_dir = args.dst_dir;
    let only_show_new_filename = args.only_show_new_filename;

    let new_filename = new_filename(&path, dst_dir.as_ref()).map_err(|e| match e.downcast::<rename_for_linux_limit::Error>() {
        Ok(rename_for_linux_limit::Error::FilenameNotFound(path)) => Error::FilenameNotFound(path),
        Err(e) => Error::UnknownError(e),
    })?;

    if only_show_new_filename {
        println!("{}", new_filename);
        return Ok(());
    }

    let new_path = if let Some(dst_dir) = dst_dir {
        fs::create_dir_all(&dst_dir)?;
        dst_dir.join(&new_filename)
    } else {
        path.with_file_name(&new_filename)
    };

    if jdt::eq_files(&path, &new_path)? {
        log::info!("Filename is already short enough: {}", new_path.display());
    } else {
        log::info!("Renamed: {} -> {}", path.display(), new_path.display());
        jdt::rename_file(&path, &new_path).map_err(|e| Error::RenameError(path, new_path, e))?;
    }

    Ok(())
}

