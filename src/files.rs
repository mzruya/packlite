use std::{ffi::OsStr, path::PathBuf};

use jwalk::WalkDir;
use rayon::iter::{ParallelBridge, ParallelIterator};

pub struct PackageFiles {
    pub package_root: PathBuf,
    pub package_file_path: PathBuf,
    pub ruby_file_paths: Vec<PathBuf>,
}

pub fn all(path: &str) -> Vec<PackageFiles> {
    let file_paths = get_file_paths(path);

    let mut ruby_files: Vec<PathBuf> = Vec::new();
    let mut package_files: Vec<PathBuf> = Vec::new();

    for file_path in file_paths {
        match file_path {
            FilePath::Ruby(path) => ruby_files.push(path),
            FilePath::Package(path) => package_files.push(path),
        }
    }

    package_files
        .iter()
        .par_bridge()
        .map(|package_file_path| {
            let package_root = package_file_path.parent().unwrap();

            let ruby_file_paths: Vec<PathBuf> = ruby_files
                .iter()
                .filter(|ruby_file_path| ruby_file_path.starts_with(package_root))
                .cloned()
                .collect();

            PackageFiles {
                package_root: package_root.to_owned(),
                package_file_path: package_file_path.to_owned(),
                ruby_file_paths,
            }
        })
        .collect()
}

#[derive(Debug)]
enum FilePath {
    Ruby(PathBuf),
    Package(PathBuf),
}

fn get_file_paths(root_path: &str) -> Vec<FilePath> {
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
