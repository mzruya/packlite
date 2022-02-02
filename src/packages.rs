use std::path::{Path, PathBuf};

use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    ast_parser::{self, ParsedFile},
    constant::{Definition, Reference},
    files::FilePath,
};

#[derive(Serialize, Deserialize)]
struct SerializablePackage {
    enforce_dependencies: bool,
    enforce_privacy: bool,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Packages {
    pub packages: Vec<Package>,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub enforce_dependencies: bool,
    pub enforce_privacy: bool,
    pub dependencies: Vec<String>,
}

fn package_name(project_root: &Path, package_root: &Path) -> String {
    let absolute_package_root = std::fs::canonicalize(package_root).unwrap();
    let absolute_project_root = std::fs::canonicalize(project_root).unwrap();

    absolute_package_root
        .strip_prefix(&absolute_project_root)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

#[instrument(skip_all)]
pub fn build(file_paths: Vec<FilePath>, project_root: &Path) -> Packages {
    let mut ruby_file_paths: Vec<PathBuf> = Vec::new();
    let mut package_file_paths: Vec<PathBuf> = Vec::new();

    for file_path in file_paths {
        match file_path {
            FilePath::Ruby(path) => ruby_file_paths.push(path),
            FilePath::Package(path) => package_file_paths.push(path),
        }
    }

    let (definitions, references) = parse_ruby_files(&ruby_file_paths);
    debug!("parse_ruby_files(&ruby_file_paths)");

    let packages: Vec<Package> = package_file_paths
        .into_iter()
        .par_bridge()
        .map(|package_file_path| {
            let package_root = package_file_path.parent().unwrap().to_owned();
            let text = std::fs::read_to_string(&package_file_path).unwrap();
            let package: SerializablePackage = serde_yaml::from_str(&text).unwrap();

            Package {
                name: package_name(project_root, &package_root),
                root: package_file_path.parent().unwrap().to_owned(),
                enforce_dependencies: package.enforce_dependencies,
                enforce_privacy: package.enforce_privacy,
                dependencies: package.dependencies.unwrap_or_default(),
            }
        })
        .collect();
    debug!("package_files grouped");

    Packages {
        packages,
        definitions,
        references,
    }
}

#[instrument(skip_all)]
fn parse_ruby_files(ruby_files: &[PathBuf]) -> (Vec<Definition>, Vec<Reference>) {
    let parsed_files: Vec<ParsedFile> = ruby_files
        .iter()
        .par_bridge()
        .map(|path| ast_parser::parse_ast(path))
        .collect();
    debug!("ast_parser::parse_ast(path)");

    let mut definitions: Vec<Definition> = Vec::new();
    let mut references: Vec<Reference> = Vec::new();

    for mut parsed_file in parsed_files {
        definitions.append(&mut parsed_file.definitions);
        references.append(&mut parsed_file.references);
    }
    debug!("flattened file paths");

    (definitions, references)
}
