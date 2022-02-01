use std::{ffi::OsStr, path::PathBuf};

use jwalk::WalkDir;
use rayon::iter::ParallelIterator;
use tracing::instrument;

#[derive(Debug)]
pub enum FilePath {
    Ruby(PathBuf),
    Package(PathBuf),
}

#[instrument]
pub fn all(root_path: &str) -> Vec<FilePath> {
    WalkDir::new(root_path)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();

            if entry.file_type().is_dir() {
                return None;
            }

            let path = entry.path();

            if path.extension().unwrap_or_else(|| OsStr::new("")) == "rb" {
                Some(FilePath::Ruby(path))
            } else if path.file_name().unwrap() == "package.yml" {
                Some(FilePath::Package(path))
            } else {
                None
            }
        })
        .collect()
}
