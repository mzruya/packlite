use crate::{
    dependency_graph::DependencyGraph,
    packages::{Definition, Package, Reference},
};

pub struct PackageValidator<'a> {
    dependency_graph: DependencyGraph<'a>,
}

#[derive(Debug)]
pub enum ViolationType {
    Inbound,
    Outbound,
}

#[derive(Debug)]
pub struct Violation {
    package_name: String,
    violation_type: ViolationType,
    from: Reference,
    to: Definition,
}

pub fn build(dependency_graph: DependencyGraph) -> PackageValidator {
    PackageValidator { dependency_graph }
}

impl<'a> PackageValidator<'a> {
    pub fn validate_all(&self) -> Vec<Violation> {
        self.dependency_graph
            .packages
            .iter()
            .flat_map(|package| self.validate(package))
            .collect()
    }

    pub fn validate(&self, package: &Package) -> Vec<Violation> {
        // let incoming_references = self.dependency_graph.incoming_references(package.id);
        let outgoing_references = self.dependency_graph.outgoing_references(package.id);

        outgoing_references
            .iter()
            .filter_map(|outgoing_reference| {
                if package.dependencies.contains(&outgoing_reference.to_package) {
                    None
                } else {
                    Some(Violation {
                        package_name: package.name.clone(),
                        violation_type: ViolationType::Outbound,
                        from: outgoing_reference.from.clone(),
                        to: outgoing_reference.to.clone(),
                    })
                }
            })
            .collect()
    }
}
