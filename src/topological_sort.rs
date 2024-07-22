use crate::{common::*, virdant_error};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub fn topological_sort<T: Eq + Hash + Clone + std::fmt::Debug>(graph: &HashMap<T, Vec<T>>) -> VirdantResult<Vec<T>> {
    let mut visited: HashSet<T> = HashSet::new();
    let mut result = vec![];

    let mut bottom_nodes = bottoms(graph, &visited);
    while bottom_nodes.len() > 0 {
        for bottom in bottom_nodes.iter() {
            visited.insert(bottom.clone());
        }

        result.extend(bottom_nodes);

        bottom_nodes = bottoms(graph, &visited);
    }

    if visited.len() < graph.len() {
        Err(virdant_error!("Cycle detected"))
    } else {
        Ok(result)
    }
}

fn bottoms<T: Eq + Hash + Clone + std::fmt::Debug>(
    graph: &HashMap<T, Vec<T>>,
    visited: &HashSet<T>,
) -> Vec<T> {
    let mut bottoms: Vec<T> = vec![];

    for (node, deps) in graph.iter() {
        if !visited.contains(node) {
            let unmet_deps: Vec<_> = deps.iter().filter(|dep| !visited.contains(dep)).collect();
            if unmet_deps.len() == 0 {
                bottoms.push(node.clone());
            }
        }
    }

    bottoms
}
