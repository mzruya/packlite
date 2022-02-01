use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;
use petgraph::{graphmap::GraphMap, Directed, Direction};
use tracing::instrument;

use crate::{constant::Definition, reference_resolver::ResolvedReference};

#[derive(Debug)]
struct NodeIndex {
    max_node_id: usize,
    node_to_id: HashMap<Node, usize>,
    id_to_node: HashMap<usize, Node>,
}

impl NodeIndex {
    pub fn new() -> Self {
        NodeIndex {
            max_node_id: 0,
            node_to_id: HashMap::new(),
            id_to_node: HashMap::new(),
        }
    }

    pub fn get_id_for_node(&self, node: &Node) -> Option<usize> {
        self.node_to_id.get(node).copied()
    }

    pub fn get_node_for_id(&self, id: usize) -> Option<&Node> {
        self.id_to_node.get(&id)
    }

    pub fn index(&mut self, node: &Node) -> usize {
        let max_node_id = &mut self.max_node_id;

        let node_id = *self.node_to_id.entry(node.clone()).or_insert_with(|| {
            let node_id = *max_node_id;
            *max_node_id += 1;
            node_id
        });

        self.id_to_node.insert(node_id, node.clone());

        node_id
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
enum NodeType {
    Definition,
    Reference,
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub struct Loc {
    begin: usize,
    end: usize,
}

impl From<lib_ruby_parser::Loc> for Loc {
    fn from(loc: lib_ruby_parser::Loc) -> Self {
        Self {
            begin: loc.begin,
            end: loc.end,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq)]
struct Node {
    pub node_type: NodeType,
    pub path: PathBuf,
    pub name: String,
    pub loc: Loc,
}

impl From<ResolvedReference> for Node {
    fn from(reference: ResolvedReference) -> Self {
        Self {
            node_type: NodeType::Reference,
            path: reference.reference.path,
            name: reference.name,
            loc: reference.reference.loc.into(),
        }
    }
}

impl From<Definition> for Node {
    fn from(definition: Definition) -> Self {
        let name = definition.qualified();

        Self {
            node_type: NodeType::Definition,
            path: definition.path,
            loc: definition.loc.into(),
            name,
        }
    }
}

pub struct ReferenceGraph {
    graph: GraphMap<usize, (), Directed>,
    node_index: NodeIndex,
    definitions: HashMap<String, Vec<Node>>,
    references: HashMap<String, Vec<Node>>,
}

#[instrument(skip_all)]
pub fn build_reference_graph(definitions: Vec<Definition>, references: Vec<ResolvedReference>) -> ReferenceGraph {
    let mut graph = GraphMap::<usize, (), Directed>::with_capacity(references.len(), references.len());

    let mut node_index: NodeIndex = NodeIndex::new();

    let definition_nodes: Vec<Node> = definitions.into_iter().map(|definition| definition.into()).collect();
    let reference_nodes: Vec<Node> = references.into_iter().map(|reference| reference.into()).collect();

    for definition_node in &definition_nodes {
        let node = node_index.index(definition_node);
        graph.add_node(node);
    }

    let definitions: HashMap<String, Vec<Node>> = definition_nodes
        .into_iter()
        .into_group_map_by(|definition| definition.name.clone());

    for reference_node in &reference_nodes {
        let node = node_index.index(reference_node);
        graph.add_node(node);

        if let Some(definitions) = definitions.get(&reference_node.name) {
            for definition in definitions {
                let from = node;
                let to = node_index.index(definition);

                graph.add_edge(from, to, ());
            }
        }
    }

    let references: HashMap<String, Vec<Node>> = reference_nodes
        .into_iter()
        .into_group_map_by(|reference| reference.name.clone());

    ReferenceGraph {
        graph,
        node_index,
        definitions,
        references,
    }
}

pub struct Usage {
    pub name: String,
    pub path: PathBuf,
    pub loc: Loc,
}

impl ReferenceGraph {
    #[instrument(skip(self))]
    pub fn find_usages(&self, name: &str) -> Vec<Usage> {
        let mut usages: Vec<Usage> = Vec::new();
        let empty_vec = Vec::new();

        let definitions = self.definitions.get(name).unwrap_or(&empty_vec);

        for definition in definitions {
            let reference_edges = self.graph.edges_directed(
                self.node_index.get_id_for_node(definition).unwrap(),
                Direction::Incoming,
            );

            for (to, _, _) in reference_edges {
                let node = self.node_index.get_node_for_id(to).unwrap();

                usages.push(Usage {
                    name: node.name.clone(),
                    path: node.path.clone(),
                    loc: node.loc.clone(),
                })
            }
        }

        usages
    }
}
