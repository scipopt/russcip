use russcip::{Node, NodeSel, Solving, prelude::*};
use std::cmp::Ordering;

/// A node selector that implements a depth-first-search (DFS) strategy.
/// It dives as deep as possible into the tree before backtracking by always
/// preferring a child of the current node, then a sibling, and only then the
/// best remaining leaf.
struct DepthFirstNodeSel;

impl NodeSel for DepthFirstNodeSel {
    fn select(&mut self, model: Model<Solving>) -> Option<Node> {
        model
            .prio_child()
            .or_else(|| model.prio_sibling())
            .or_else(|| model.best_leaf())
    }

    fn comp(&mut self, node1: Node, node2: Node) -> Ordering {
        // Order deeper nodes first; break ties by the smaller lower bound.
        node2.depth().cmp(&node1.depth()).then_with(|| {
            node1
                .lower_bound()
                .partial_cmp(&node2.lower_bound())
                .unwrap_or(Ordering::Equal)
        })
    }
}

fn main() {
    let mut model = Model::new()
        .include_default_plugins()
        .read_prob("data/test/p0201.mps")
        .expect("Failed to read problem file");

    // Add the custom node selector. By default it has the highest priority,
    // so SCIP uses it over the built-in node selectors.
    model.add(
        nodesel(DepthFirstNodeSel)
            .name("DepthFirst")
            .desc("Depth-first-search node selector"),
    );

    let solved = model.solve();
    assert_eq!(solved.status(), Status::Optimal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_first_node_selection() {
        main();
    }
}
