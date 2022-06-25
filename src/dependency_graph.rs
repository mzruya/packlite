use std::collections::HashMap;

use itertools::Itertools;
use petgraph::{graphmap::GraphMap, Directed, Direction};
use tracing::instrument;

use crate::packages::{Definition, DefinitionId, Package, PackageId, Reference, ReferenceId};

pub struct DependencyGraph<'a> {
    graph: GraphMap<PackageId, (ReferenceId, DefinitionId), Directed>,
    pub packages: &'a [Package],
    pub id_to_package: HashMap<PackageId, &'a Package>,
    pub id_to_definition: HashMap<DefinitionId, &'a Definition>,
    pub id_to_reference: HashMap<ReferenceId, &'a Reference>,
}

#[instrument(skip_all)]
pub fn build(packages: &[Package]) -> DependencyGraph {
    let reference_count = packages.iter().map(|package| package.references.len()).sum();

    let mut graph = GraphMap::<PackageId, (ReferenceId, DefinitionId), Directed>::with_capacity(packages.len(), reference_count);

    let mut definitions_by_qualified_name: HashMap<&str, Vec<(DefinitionId, PackageId)>> = HashMap::new();
    for package in packages.iter() {
        for definition in &package.definitions {
            definitions_by_qualified_name.entry(&definition.name).or_insert_with(Vec::new).push((definition.id, package.id));
        }
    }

    for package in packages.iter() {
        graph.add_node(package.id);

        for reference in &package.references {
            let definitions = definitions_by_qualified_name.get(&reference.name as &str);

            if let Some(definitions) = definitions {
                for (definition_id, definition_package_id) in definitions {
                    if &package.id != definition_package_id {
                        graph.add_edge(package.id, *definition_package_id, (reference.id, *definition_id));
                    }
                }
            }
        }
    }

    let id_to_packages: HashMap<PackageId, Vec<&Package>> = packages.iter().into_group_map_by(|package| package.id);
    let id_to_definitions: HashMap<DefinitionId, Vec<&Definition>> = packages.iter().flat_map(|package| &package.definitions).into_group_map_by(|defintion| defintion.id);
    let id_to_references: HashMap<ReferenceId, Vec<&Reference>> = packages.iter().flat_map(|package| &package.references).into_group_map_by(|reference| reference.id);

    let id_to_package = flatten_map(id_to_packages);
    let id_to_definition = flatten_map(id_to_definitions);
    let id_to_reference = flatten_map(id_to_references);

    DependencyGraph {
        graph,
        packages,
        id_to_package,
        id_to_definition,
        id_to_reference,
    }
}

fn flatten_map<Key: std::fmt::Debug + std::hash::Hash + Eq, Value>(map: HashMap<Key, Vec<Value>>) -> HashMap<Key, Value> {
    map.into_iter()
        .map(|(k, mut v)| {
            if v.len() != 1 {
                panic!("{k:?} has duplicates");
            }

            (k, v.pop().unwrap())
        })
        .collect()
}
pub struct Edge {
    pub from_package: String,
    pub to_package: String,
    pub from: Reference,
    pub to: Definition,
}

impl<'a> DependencyGraph<'a> {
    pub fn incoming_references(&self, package_id: PackageId) -> Vec<Edge> {
        let _edges = self.graph.edges_directed(package_id, Direction::Incoming);

        todo!()
    }

    pub fn outgoing_references(&self, package_id: PackageId) -> Vec<Edge> {
        let edges = self.graph.edges_directed(package_id, Direction::Outgoing);

        edges
            .map(|(from_package_id, to_package_id, (reference_id, definition_id))| {
                let from_package = *self.id_to_package.get(&from_package_id).unwrap();
                let to_package = *self.id_to_package.get(&to_package_id).unwrap();
                let from = *self.id_to_reference.get(reference_id).unwrap();
                let to = *self.id_to_definition.get(definition_id).unwrap();

                Edge {
                    from_package: from_package.name.clone(),
                    to_package: to_package.name.clone(),
                    from: from.to_owned(),
                    to: to.to_owned(),
                }
            })
            .collect()
    }
}
