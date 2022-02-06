use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::iter::ParallelIterator;
use tracing::{instrument, trace};

pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub package_file: PathBuf,
    pub ruby_files: Vec<PathBuf>,
}

enum SearchBy<'a> {
    Extension(&'a str),
    FileName(&'a str),
}

#[instrument(skip_all)]
pub fn all(root_path: &Path) -> Vec<Package> {
    let package_files = walkdir(root_path, SearchBy::FileName("package.yml"));

    package_files
        .into_iter()
        .filter_map(|package_file| {
            let package_root = package_file.parent().unwrap();

            let ruby_files = walkdir(package_root, SearchBy::Extension("rb"));

            let absolute_package_root = std::fs::canonicalize(package_root).unwrap();
            let absolute_project_root = std::fs::canonicalize(root_path).unwrap();
            let package_name = absolute_package_root
                .strip_prefix(&absolute_project_root)
                .unwrap()
                .to_string_lossy()
                .to_string();

            // Not supporting the root package.
            if package_name.is_empty() {
                return None;
            }

            trace!("{}: found {} ruby files in {:?}", package_name, ruby_files.len(), &package_root);

            Some(Package {
                name: if package_name.is_empty() {
                    "root".to_string()
                } else {
                    package_name
                },
                root: package_root.to_owned(),
                package_file,
                ruby_files,
            })
        })
        .collect()
}

fn walkdir(root_path: &Path, search: SearchBy) -> Vec<PathBuf> {
    WalkDir::new(root_path)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();

            if entry.file_type().is_dir() {
                return None;
            }

            let path = entry.path();

            let search_match = match search {
                SearchBy::Extension(extension) => path.extension().unwrap_or_else(|| OsStr::new("")) == extension,
                SearchBy::FileName(file_name) => path.file_name().unwrap() == file_name,
            };

            if search_match {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}
