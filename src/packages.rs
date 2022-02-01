use std::path::PathBuf;

use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{
    ast_parser::{self, ParsedFile},
    constant::{Definition, Reference},
    files::PackageFiles,
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
    pub root: PathBuf,
    pub enforce_dependencies: bool,
    pub enforce_privacy: bool,
    pub dependencies: Vec<String>,
    pub ruby_file_paths: Vec<PathBuf>,
}

pub fn build(package_files: Vec<PackageFiles>) -> Packages {
    let ruby_file_paths: Vec<&PathBuf> = package_files
        .iter()
        .flat_map(|package_files| &package_files.ruby_file_paths)
        .collect();
    let (definitions, references) = parse_ruby_files(&ruby_file_paths);

    let packages: Vec<Package> = package_files
        .into_iter()
        .par_bridge()
        .map(|package_files| {
            let text = std::fs::read_to_string(package_files.package_file_path).unwrap();
            let package: SerializablePackage = serde_yaml::from_str(&text).unwrap();

            Package {
                root: package_files.package_root,
                enforce_dependencies: package.enforce_dependencies,
                enforce_privacy: package.enforce_privacy,
                dependencies: package.dependencies.unwrap_or_default(),
                ruby_file_paths: package_files.ruby_file_paths,
            }
        })
        .collect();

    Packages {
        packages,
        definitions,
        references,
    }
}

fn parse_ruby_files(ruby_files: &[&PathBuf]) -> (Vec<Definition>, Vec<Reference>) {
    let parsed_files: Vec<ParsedFile> = ruby_files
        .iter()
        .par_bridge()
        .map(|path| ast_parser::parse_ast(path))
        .collect();

    let mut definitions: Vec<Definition> = Vec::new();
    let mut references: Vec<Reference> = Vec::new();

    for mut parsed_file in parsed_files {
        definitions.append(&mut parsed_file.definitions);
        references.append(&mut parsed_file.references);
    }

    (definitions, references)
}
