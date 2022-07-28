mod constant;
mod parser;
mod reference_resolver;
mod visitor;

pub use constant::{Constant, Loc};
pub use parser::parse as parse_ast;
pub use parser::ParsedFile;
pub use reference_resolver::{resolve, ResolvedReference};

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn remove_absolute_paths(string: String) -> String {
        let current_dir = std::env::current_dir().unwrap();
        string.replace(current_dir.to_str().unwrap(), ".")
    }

    fn test_fixture(ruby_file_path: &str, expectation_file_path: &str) {
        let parsed_file = super::parse_ast(Path::new("./"), Path::new(ruby_file_path));

        let actual = remove_absolute_paths(format!("{parsed_file:#?}"));

        if std::env::var("OVERWRITE_FIXTURES").is_ok() {
            std::fs::write(expectation_file_path, &actual).unwrap();
        }

        let expected = std::fs::read_to_string(expectation_file_path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_fixtures() {
        let examples = [
            ("./fixtures/nested_classes.rb", "./fixtures/nested_classes_parsed.output"),
            ("./fixtures/root_reference.rb", "./fixtures/root_reference_parsed.output"),
        ];

        for (ruby_file_path, expectation_file_path) in examples {
            test_fixture(ruby_file_path, expectation_file_path)
        }
    }
}
