use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::constant::{Definition, Reference};

#[derive(Debug, Clone)]
pub struct ResolvedReference {
    pub name: String,
    pub reference: Reference,
}

pub fn resolve(definitions: &[Definition], references: &[Reference]) -> Vec<ResolvedReference> {
    let definition_by_qualified_name = definitions
        .iter()
        .into_group_map_by(|definition| definition.qualified());

    references
        .iter()
        .par_bridge()
        .filter_map(|reference| {
            let name = reference
                .nestings()
                .into_iter()
                .find(|nesting| definition_by_qualified_name.contains_key(nesting));

            name.map(|name| ResolvedReference {
                name,
                reference: reference.clone(),
            })
        })
        .collect()
}
