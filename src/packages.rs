use std::path::PathBuf;

use lib_ruby_parser::Loc;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};

use crate::ast::{self, ParsedFile};

#[derive(Serialize, Deserialize)]
struct SerializablePackage {
    enforce_dependencies: bool,
    enforce_privacy: bool,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DefinitionId(usize);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct PackageId(usize);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ReferenceId(usize);

impl PackageId {
    fn new() -> Self {
        let uid = uid::Id::<usize>::new();
        Self(uid.get())
    }
}

impl ReferenceId {
    fn new() -> Self {
        let uid = uid::Id::<usize>::new();
        Self(uid.get())
    }
}

impl DefinitionId {
    fn new() -> Self {
        let uid = uid::Id::<usize>::new();
        Self(uid.get())
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub id: ReferenceId,
    pub package_id: PackageId,
    pub name: String,
    pub loc: Loc,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub id: DefinitionId,
    pub package_id: PackageId,
    pub name: String,
    pub loc: Loc,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub id: PackageId,
    pub name: String,
    pub root: PathBuf,
    pub enforce_dependencies: bool,
    pub enforce_privacy: bool,
    pub dependencies: Vec<String>,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

#[instrument(skip_all)]
pub fn parse(packages: Vec<crate::files::Package>) -> Vec<Package> {
    packages
        .into_iter()
        .par_bridge()
        .map(|package| {
            let package_id = PackageId::new();

            let (definitions, references) = parse_ruby_files(&package.ruby_files);
            let text = std::fs::read_to_string(&package.package_file).unwrap();
            let package_yaml: SerializablePackage = serde_yaml::from_str(&text).unwrap();

            let definitions = definitions
                .into_iter()
                .map(|definition| Definition {
                    id: DefinitionId::new(),
                    package_id,
                    name: definition.name,
                    loc: definition.loc,
                    path: definition.path,
                })
                .collect();

            let references = references
                .into_iter()
                .map(|reference| Reference {
                    id: ReferenceId::new(),
                    package_id,
                    name: reference.name,
                    loc: reference.loc,
                    path: reference.path,
                })
                .collect();

            Package {
                id: PackageId::new(),
                name: package.name,
                root: package.root,
                enforce_dependencies: package_yaml.enforce_dependencies,
                enforce_privacy: package_yaml.enforce_privacy,
                dependencies: package_yaml.dependencies.unwrap_or_default(),
                definitions,
                references,
            }
        })
        .collect()
}

#[instrument(skip_all)]
fn parse_ruby_files(ruby_files: &[PathBuf]) -> (Vec<ast::Definition>, Vec<ast::ResolvedReference>) {
    trace!("ast_parser::parse_ast");
    let parsed_files: Vec<ParsedFile> = ruby_files.iter().par_bridge().map(|path| ast::parse_ast(path)).collect();

    let mut definitions: Vec<ast::Definition> = Vec::new();
    let mut references: Vec<ast::Reference> = Vec::new();

    for mut parsed_file in parsed_files {
        definitions.append(&mut parsed_file.definitions);
        references.append(&mut parsed_file.references);
    }

    // Resolves ruby constant references to the fully qualified constant they refer to.
    trace!("reference_resolver::resolve()",);
    let references = ast::resolve(&definitions, &references);

    (definitions, references)
}
