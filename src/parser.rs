use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;


use crate::{
    ast::{self, Loc},
    files, resolver,
};

#[derive(Debug, Clone, Serialize)]
pub struct Definition {
    pub package: String,
    pub name: String,
    pub public: bool,
    pub loc: Loc,
}

#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    pub package: String,
    pub name: String,
    pub loc: Loc,
}

#[derive(Serialize)]
pub struct Project {
    pub packages: Vec<files::Package>,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

pub fn parse_ruby_files(root_path: &Path, ruby_files: &[PathBuf]) -> Vec<ast::ParsedFile> {
    let root_path = std::fs::canonicalize(root_path).unwrap();
    ruby_files.iter().par_bridge().map(|path| ast::parse_ast(&root_path, path)).collect()
}

pub fn apply_package_metadata(definitions: Vec<ast::Constant>, references: Vec<resolver::ResolvedReference>, packages: Vec<files::Package>, public_path: &str, ignore_constants: &[String]) -> Project {
    let package_name_by_path: HashMap<&Path, &str> = packages.iter().map(|package| (package.root.as_ref(), package.name.as_ref())).collect();

    let definitions = definitions
        .into_iter()
        .par_bridge()
        .filter(|definition| !ignore_constants.contains(&definition.name))
        .map(|definition| {
            let package = find_package(&definition.loc.path, &package_name_by_path);

            let mut public = false;
            let mut package_name = "root".to_string();

            if let Some((path, name)) = package {
                let relative_path = definition.loc.path.strip_prefix(path).unwrap();
                public = relative_path.starts_with(public_path);
                package_name = name;
            }

            Definition {
                name: definition.qualified(),
                loc: definition.loc,
                public,
                package: package_name,
            }
        })
        .collect();

    let references = references
        .into_iter()
        .par_bridge()
        .map(|reference| {
            let package = find_package(&reference.loc.path, &package_name_by_path);
            let mut package_name = "root".to_string();

            if let Some((_path, name)) = package {
                package_name = name;
            }

            Reference {
                name: reference.name,
                package: package_name,
                loc: reference.loc,
            }
        })
        .collect();

    Project { packages, definitions, references }
}

pub fn find_package<'a>(path: &'a Path, package_name_by_path: &'a HashMap<&'a Path, &'a str>) -> Option<(&'a Path, String)> {
    let package = path.ancestors().filter_map(|ancestor| package_name_by_path.get_key_value(&ancestor)).next();
    package.map(|(path, name)| (*path, name.to_string()))
}
