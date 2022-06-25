use crate::{
    dependency_graph::DependencyGraph,
    packages::{Definition, Package, Reference},
};

pub struct PackageValidator<'a> {
    dependency_graph: DependencyGraph<'a>,
}

#[derive(Debug)]
pub enum Direction {
    Inbound,
    Outbound,
}

#[derive(Debug)]
pub enum Type {
    Privacy,
    Depdendency,
}

#[derive(Debug)]
pub struct Violation {
    package_name: String,
    from_package: String,
    to_package: String,
    direction: Direction,
    r#type: Type,
    from: Reference,
    to: Definition,
}

pub trait Validator {
    fn validate(&self, dependency_graph: &DependencyGraph, package: &Package) -> Vec<Violation>;
}

pub struct OutgoingReferenceValidator;

impl<'a> Validator for OutgoingReferenceValidator {
    fn validate(&self, dependency_graph: &DependencyGraph, package: &Package) -> Vec<Violation> {
        let outgoing_references = dependency_graph.outgoing_references(package.id);

        outgoing_references
            .iter()
            .filter_map(|outgoing_reference| {
                if package.dependencies.contains(&outgoing_reference.to_package) {
                    return None;
                }

                Some(Violation {
                    package_name: package.name.clone(),
                    from_package: outgoing_reference.from_package.clone(),
                    to_package: outgoing_reference.to_package.clone(),
                    direction: Direction::Outbound,
                    r#type: Type::Depdendency,
                    from: outgoing_reference.from.clone(),
                    to: outgoing_reference.to.clone(),
                })
            })
            .collect()
    }
}

pub fn all_validators() -> Vec<Box<dyn Validator>> {
    vec![Box::new(OutgoingReferenceValidator)]
}

pub fn validate_all(dependency_graph: &DependencyGraph, validators: Vec<Box<dyn Validator>>) -> Vec<Violation> {
    dependency_graph
        .packages
        .iter()
        .flat_map(|package| validators.iter().flat_map(|validator| validator.validate(dependency_graph, package)))
        .collect()
}
