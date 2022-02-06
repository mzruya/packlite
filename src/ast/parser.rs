use super::constant::{Definition, Reference};
use super::visitor;
use lib_ruby_parser::{traverse::visitor::Visitor, Parser, ParserOptions};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

pub fn parse(path: &Path) -> ParsedFile {
    let text = std::fs::read_to_string(path).unwrap();
    let (definitions, references) = parse_text(&text, path);

    ParsedFile {
        path: path.to_owned(),
        definitions,
        references,
    }
}

fn parse_text(text: &str, path: &Path) -> (Vec<Definition>, Vec<Reference>) {
    let parser = Parser::new(text, ParserOptions::default());
    let ast = parser.do_parse().ast;

    if ast.is_none() {
        return (Vec::new(), Vec::new());
    }

    let mut visitor = visitor::Visitor::new(path);
    visitor.visit(&ast.unwrap());

    let definitions: Vec<Definition> = visitor.definitions.into_iter().map(|constant| constant.into()).collect();

    let references: Vec<Reference> = visitor.references.into_iter().map(|constant| constant.into()).collect();

    (definitions, references)
}
