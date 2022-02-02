use lib_ruby_parser::{traverse::visitor::Visitor, Parser, ParserOptions};
use std::path::{Path, PathBuf};
mod visitor;

use crate::constant::{Definition, Reference};

#[derive(Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

pub fn parse_ast(path: &Path) -> ParsedFile {
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

    let definitions: Vec<Definition> = visitor
        .definitions
        .into_iter()
        .map(|constant| constant.into())
        .collect();
    let references: Vec<Reference> = visitor.references.into_iter().map(|constant| constant.into()).collect();

    (definitions, references)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn remove_absolute_paths(string: String) -> String {
        let current_dir = std::env::current_dir().unwrap();
        string.replace(current_dir.to_str().unwrap(), ".")
    }

    fn test_fixture(ruby_file_path: &str, expectation_file_path: &str) {
        let parsed_file = super::parse_ast(Path::new(ruby_file_path));

        let actual = remove_absolute_paths(format!("{parsed_file:#?}"));

        if std::env::var("OVERWRITE_FIXTURES").is_ok() {
            std::fs::write(expectation_file_path, &actual).unwrap();
        }

        let expected = std::fs::read_to_string(expectation_file_path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_fixtures() {
        let examples = [("./fixtures/nested_classes.rb", "./fixtures/nested_classes.output")];

        for (ruby_file_path, expectation_file_path) in examples {
            test_fixture(ruby_file_path, expectation_file_path)
        }
    }
}
