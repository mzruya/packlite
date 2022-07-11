use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Serialize, Deserialize)]
struct SerializablePackage {
    enforce_dependencies: bool,
    enforce_privacy: bool,
    dependencies: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub enforce_dependencies: bool,
    pub enforce_privacy: bool,
    pub dependencies: Option<Vec<String>>,
}

enum SearchBy<'a> {
    Extension(&'a str),
    FileName(&'a str),
}

#[instrument(skip_all)]
pub fn all_ruby_files(root_path: &Path, package_paths: &[PathBuf]) -> Vec<PathBuf> {
    let ruby_files: Vec<PathBuf> = paths_to_scan(root_path, package_paths).iter().flat_map(|path| walkdir(path, SearchBy::Extension("rb"))).collect();
    ruby_files.into_iter().par_bridge().map(|file| std::fs::canonicalize(file).unwrap()).collect()
}

#[instrument(skip_all)]
pub fn all_packages(root_path: &Path, package_paths: &[PathBuf]) -> Vec<Package> {
    let package_files: Vec<PathBuf> = paths_to_scan(root_path, package_paths)
        .iter()
        .flat_map(|path| walkdir(path, SearchBy::FileName("package.yml")))
        .collect();

    package_files
        .into_iter()
        .par_bridge()
        .filter_map(|package_file| {
            let package_root = package_file.parent().unwrap();
            let absolute_package_root = std::fs::canonicalize(package_root).unwrap();
            let absolute_project_root = std::fs::canonicalize(root_path).unwrap();
            let package_name = absolute_package_root.strip_prefix(&absolute_project_root).unwrap().to_string_lossy().to_string();
            let package_yaml: SerializablePackage = serde_yaml::from_str(&std::fs::read_to_string(package_file).unwrap()).unwrap();

            Some(Package {
                name: if package_name.is_empty() { "root".to_string() } else { package_name },
                root: absolute_package_root,
                enforce_dependencies: package_yaml.enforce_dependencies,
                enforce_privacy: package_yaml.enforce_privacy,
                dependencies: package_yaml.dependencies,
            })
        })
        .collect()
}

fn paths_to_scan(root_path: &Path, package_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths_to_scan: Vec<PathBuf> = Vec::new();

    if package_paths.is_empty() {
        paths_to_scan.push(root_path.to_owned())
    } else {
        for package_path in package_paths {
            paths_to_scan.push(root_path.join(package_path))
        }
    }

    paths_to_scan
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
