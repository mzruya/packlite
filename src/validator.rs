use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    files::Package,
    parser::{self, Definition, Reference},
};

#[derive(Deserialize, Serialize, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub enum ViolationType {
    #[serde(rename = "dependency")]
    Dependency,
    #[serde(rename = "privacy")]
    Privacy,
}

#[derive(Serialize)]
pub struct Violation {
    violation_type: ViolationType,
    violated_pack: String,
    violating_pack: String,
    definition: Definition,
    reference: Reference,
}

struct ValidationContext<'a> {
    definition_by_name: HashMap<&'a String, Vec<&'a Definition>>,
    definition_by_package: HashMap<&'a String, Vec<&'a Definition>>,
    reference_by_name: HashMap<&'a String, Vec<&'a Reference>>,
    reference_by_package: HashMap<&'a String, Vec<&'a Reference>>,
    package_by_name: HashMap<&'a String, &'a Package>,
}

#[derive(Serialize, Deserialize)]
pub struct DeprecatedReference {
    pub violations: Vec<ViolationType>,
    pub files: Vec<PathBuf>,
}

pub struct DeprecatedReferences {
    pub violating_pack: String,
    pub deprecated_references: BTreeMap<String, BTreeMap<String, DeprecatedReference>>,
}

impl<'a> ValidationContext<'a> {
    fn from_project(project: &'a parser::Project) -> Self {
        Self {
            definition_by_name: project.definitions.iter().into_grouping_map_by(|definition| &definition.name).collect(),
            definition_by_package: project.definitions.iter().into_grouping_map_by(|definition| &definition.package).collect(),
            reference_by_name: project.references.iter().into_grouping_map_by(|reference| &reference.name).collect(),
            reference_by_package: project.references.iter().into_grouping_map_by(|reference| &reference.package).collect(),
            package_by_name: project.packages.iter().map(|package| (&package.name, package)).collect(),
        }
    }

    fn package(&self, package: &String) -> &Package {
        self.package_by_name.get(package).unwrap()
    }

    fn all_references_in_package(&self, package: &String) -> Option<Vec<&Reference>> {
        self.reference_by_package.get(package).map(Vec::to_owned)
    }

    fn all_definitions_in_package(&self, package: &String) -> Option<Vec<&Definition>> {
        self.definition_by_package.get(package).map(Vec::to_owned)
    }

    fn all_definitions_for(&self, name: &String) -> Option<Vec<&Definition>> {
        self.definition_by_name.get(name).map(Vec::to_owned)
    }

    fn packages_with_definition(&self, definition: &String) -> Vec<&String> {
        self.definition_by_name.get(definition).unwrap().iter().map(|definition| &definition.package).unique().collect()
    }

    fn packages_referencing(&self, definition: &String) -> Vec<&String> {
        self.reference_by_name.get(definition).unwrap().iter().map(|reference| &reference.package).unique().collect()
    }
}

pub fn validate(project: &parser::Project) -> Vec<Violation> {
    let validation_context = ValidationContext::from_project(project);
    project
        .packages
        .iter()
        .flat_map(|package| {
            let mut violations = vec![];

            violations.append(&mut privacy_violation(package, &validation_context));
            violations.append(&mut dependency_violation(package, &validation_context));

            violations
        })
        .filter(|violation| violation.violated_pack != "root")
        .collect()
}

pub fn deprecated_references(violations: &[Violation]) -> Vec<DeprecatedReferences> {
    violations
        .iter()
        .into_group_map_by(|violation| &violation.violating_pack)
        .into_iter()
        .map(|(violating_pack, violations)| DeprecatedReferences {
            violating_pack: violating_pack.to_owned(),
            deprecated_references: deprecated_references_for_pack(&violations),
        })
        .collect()
}

fn deprecated_references_for_pack(violations: &[&Violation]) -> BTreeMap<String, BTreeMap<String, DeprecatedReference>> {
    violations
        .iter()
        .into_group_map_by(|violation| &violation.violated_pack)
        .into_iter()
        .sorted_by_key(|(violated_pack, _)| violated_pack.clone())
        .map(|(violated_pack, violations)| {
            let pack_violations = violations
                .into_iter()
                .into_group_map_by(|violation| format!("::{}", violation.definition.name))
                .into_iter()
                .sorted_by_key(|(constant, _)| constant.clone())
                .map(|(constant, violations)| {
                    let deprecated_reference = DeprecatedReference {
                        violations: violations.iter().map(|violation| violation.violation_type.clone()).sorted().unique().collect_vec(),
                        files: violations.iter().map(|violation| violation.reference.loc.relative_path().to_owned()).sorted().unique().collect_vec(),
                    };

                    (constant, deprecated_reference)
                })
                .collect();

            (violated_pack.to_owned(), pack_violations)
        })
        .collect()
}

fn privacy_violation(package: &Package, validation_context: &ValidationContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for reference in validation_context.all_references_in_package(&package.name).unwrap_or_default() {
        let definitions = validation_context.all_definitions_for(&reference.name).unwrap_or_default();

        // a namespace constant tends to be re-defined many times, sometimes as a public constant and sometimes as private
        // to avoid false positives, if it has a single definition that's public, treat all have them as allowed
        if definitions.iter().any(|definition| definition.public) {
            continue;
        }

        let private_definitions = definitions.iter().filter(|definition| {
            if !validation_context.package(&definition.package).enforce_privacy {
                return false;
            }

            !definition.public && definition.package != package.name
        });

        for definition in private_definitions {
            violations.push(Violation {
                violation_type: ViolationType::Privacy,
                violated_pack: definition.package.clone(),
                violating_pack: package.name.clone(),
                definition: (*definition).to_owned(),
                reference: reference.to_owned(),
            })
        }
    }

    violations
}

fn dependency_violation(package: &Package, validation_context: &ValidationContext) -> Vec<Violation> {
    let mut violations = Vec::new();

    for reference in validation_context.all_references_in_package(&package.name).unwrap_or_default() {
        let definitions = validation_context.all_definitions_for(&reference.name).unwrap_or_default();
        let dependency_violations = definitions.iter().filter(|definition| {
            if !validation_context.package(&definition.package).enforce_dependencies {
                return false;
            }

            if definition.package == package.name {
                return false;
            }

            if let Some(dependencies) = &package.dependencies {
                !dependencies.contains(&definition.package)
            } else {
                true
            }
        });

        for definition in dependency_violations {
            violations.push(Violation {
                violation_type: ViolationType::Dependency,
                violated_pack: definition.package.clone(),
                violating_pack: package.name.clone(),
                definition: (*definition).to_owned(),
                reference: reference.to_owned(),
            })
        }
    }

    violations
}
