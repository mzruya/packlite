use super::constant::Constant;
use super::visitor;
use lib_ruby_parser::{traverse::visitor::Visitor, Parser, ParserOptions};
use line_col::LineColLookup;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub definitions: Vec<Constant>,
    pub references: Vec<Constant>,
}

pub fn parse(root_path: &Path, path: &Path) -> ParsedFile {
    let text = std::fs::read_to_string(path).unwrap();
    let (definitions, references) = parse_text(&text, root_path, path);

    ParsedFile {
        path: path.to_owned(),
        definitions,
        references,
    }
}

fn parse_text(text: &str, root_path: &Path, path: &Path) -> (Vec<Constant>, Vec<Constant>) {
    let parser = Parser::new(text, ParserOptions::default());
    let ast = parser.do_parse().ast;

    if ast.is_none() {
        return (Vec::new(), Vec::new());
    }

    let line_lookup = LineColLookup::new(text);
    let mut visitor = visitor::Visitor::new(root_path, path, &line_lookup);
    visitor.visit(&ast.unwrap());

    (visitor.definitions, visitor.references)
}
