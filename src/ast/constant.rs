use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Constant {
    pub scope: Option<String>,
    pub name: String,
    pub loc: Loc,
}

#[derive(Debug, Clone, Serialize)]
pub struct Loc {
    pub path: PathBuf,
    pub root_path: PathBuf,
    pub begin: CaretPos,
    pub end: CaretPos,
}

#[derive(Debug, Clone, Serialize)]
pub struct CaretPos {
    pub line: usize,
    pub column: usize,
}

impl Constant {
    pub fn nestings(&self) -> Vec<String> {
        let mut nestings = Vec::new();

        let unwrapped_scope = self.scope.clone().unwrap_or_else(|| "".to_string());
        let mut remaining_parts: Vec<&str> = unwrapped_scope.split("::").collect();

        while let Some(nesting_part) = remaining_parts.pop() {
            let mut parts: Vec<&str> = remaining_parts.clone();
            parts.push(nesting_part);
            parts.push(&self.name);
            nestings.push(parts.join("::"));
        }

        nestings.push(self.name.clone());
        nestings
    }
}

impl Constant {
    pub fn qualified(&self) -> String {
        qualified(&self.scope, &self.name)
    }
}

impl Loc {
    pub fn relative_path(&self) -> &Path {
        self.path.strip_prefix(&self.root_path).unwrap()
    }
}

fn qualified(scope: &Option<String>, name: &str) -> String {
    if let Some(scope) = scope {
        format!("{}::{}", scope, name)
    } else {
        name.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{CaretPos, Constant, Loc};
    use std::{path::PathBuf, str::FromStr};

    fn constant() -> Constant {
        Constant {
            scope: Some("A::B::C".to_owned()),
            name: "InC".to_owned(),
            loc: Loc {
                path: PathBuf::from_str("./fixtures/nested_classes.rb").unwrap(),
                root_path: PathBuf::from_str("./").unwrap(),
                begin: CaretPos { line: 1, column: 1 },
                end: CaretPos { line: 1, column: 1 },
            },
        }
    }

    #[test]
    fn test_qualified() {
        assert_eq!(constant().qualified(), "A::B::C::InC".to_owned());
    }

    #[test]
    fn test_nestings() {
        assert_eq!(constant().nestings(), vec!["A::B::C::InC", "A::B::InC", "A::InC", "InC"].to_owned());
    }
}
