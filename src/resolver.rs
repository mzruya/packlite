mod ast_resolver;

use crate::ast::{self, Loc};
use serde::Serialize;


#[derive(Debug, Clone, Serialize)]
pub struct ResolvedReference {
    pub name: String,
    pub loc: Loc,
}


pub fn resolve_references(parsed_files: Vec<ast::ParsedFile>) -> (Vec<ast::Constant>, Vec<ResolvedReference>) {
    let mut definitions: Vec<ast::Constant> = Vec::new();
    let mut references: Vec<ast::Constant> = Vec::new();

    for mut parsed_file in parsed_files {
        definitions.append(&mut parsed_file.definitions);
        references.append(&mut parsed_file.references);
    }

    // Resolves ruby constant references to the fully qualified constant they refer to.
    let references = ast_resolver::resolve(&definitions, &references);

    (definitions, references)
}
