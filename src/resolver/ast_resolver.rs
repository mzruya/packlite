use std::collections::HashMap;

use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::ResolvedReference;
use crate::ast::Constant;

pub fn resolve(definitions: &[Constant], references: &[Constant]) -> Vec<ResolvedReference> {
    let definition_by_qualified_name = definitions.iter().into_group_map_by(|definition| definition.qualified());

    let resolved_references: Vec<ResolvedReference> = references
        .iter()
        .par_bridge()
        .filter_map(|reference| resolve_reference(&definition_by_qualified_name, reference))
        .collect();

    resolved_references.into_iter().sorted_by_key(|reference| reference.loc.begin.line).collect()
}

fn resolve_reference(definition_by_qualified_name: &HashMap<String, Vec<&Constant>>, reference: &Constant) -> Option<ResolvedReference> {
    let name = &reference.name;

    if name.starts_with("::") {
        let qualified_name = name.trim_start_matches("::");

        definition_by_qualified_name.get(qualified_name).map(|_| ResolvedReference {
            name: qualified_name.to_owned(),
            loc: reference.loc.clone(),
        })
    } else {
        let name = reference.nestings().into_iter().find(|nesting| definition_by_qualified_name.contains_key(nesting));

        name.map(|name| ResolvedReference { name, loc: reference.loc.clone() })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn test_fixture(ruby_file_path: &str, expectation_file_path: &str) {
        let parsed_file = crate::ast::parse_ast(Path::new("./"), Path::new(ruby_file_path));
        let actual = format!("{:#?}", super::resolve(&parsed_file.definitions, &parsed_file.references));

        if std::env::var("OVERWRITE_FIXTURES").is_ok() {
            std::fs::write(expectation_file_path, &actual).unwrap();
        }

        let expected = std::fs::read_to_string(expectation_file_path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_fixtures() {
        let examples = [
            ("./fixtures/nested_classes.rb", "./fixtures/nested_classes_resolved_references.output"),
            ("./fixtures/root_reference.rb", "./fixtures/root_reference_resolved_references.output"),
        ];

        for (ruby_file_path, expectation_file_path) in examples {
            test_fixture(ruby_file_path, expectation_file_path)
        }
    }
}
