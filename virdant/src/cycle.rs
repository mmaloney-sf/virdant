use std::hash::Hash;
use indexmap::{IndexMap, IndexSet};

pub fn detect_cycle<T: Eq + Hash + Clone>(graph: &IndexMap<T, Vec<T>>) -> Result<(), Vec<T>> {
    let mut visited = IndexSet::new();
    let mut recursion_stack = Vec::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            if let Some(cycle) = dfs(graph, node, &mut visited, &mut recursion_stack) {
                return Err(cycle);
            }
        }
    }
    Ok(())
}

fn dfs<T: Clone + Eq + Hash>(
    graph: &IndexMap<T, Vec<T>>,
    node: &T,
    visited: &mut IndexSet<T>,
    recursion_stack: &mut Vec<T>,
) -> Option<Vec<T>> {
    visited.insert(node.clone());
    recursion_stack.push(node.clone());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if let Some(cycle) = dfs(graph, neighbor, visited, recursion_stack) {
                    return Some(cycle);
                }
            } else if recursion_stack.contains(neighbor) {
                // Found a cycle, extract the cycle path
                let cycle_start_index = recursion_stack.iter().position(|x| x == neighbor).unwrap();
                let cycle = recursion_stack[cycle_start_index..].to_vec();
                return Some(cycle);
            }
        }
    }

    recursion_stack.pop();
    None
}
