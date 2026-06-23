use crate::node::Node;
use crate::{Model, Solving, ffi};
use std::cmp::Ordering;

/// A trait for defining custom node selectors.
///
/// A node selector decides which of the currently open (leaf) nodes of the
/// branch-and-bound tree should be processed next. It does so through two
/// callbacks:
/// * [`select`](NodeSel::select) picks the next node to be processed, and
/// * [`comp`](NodeSel::comp) defines a total order on the open nodes, which SCIP
///   uses to keep its internal node queue sorted.
pub trait NodeSel {
    /// Selects the next node to be processed.
    ///
    /// # Arguments
    /// * `model` - the current model of the SCIP instance in `Solving` stage.
    ///
    /// # Returns
    ///
    /// The node that should be processed next, or `None` to let SCIP fall back
    /// to the node with the best [`comp`](NodeSel::comp) ranking.
    fn select(&mut self, model: Model<Solving>) -> Option<Node>;

    /// Compares two nodes to define their processing order.
    ///
    /// # Arguments
    /// * `node1` - the first node to compare.
    /// * `node2` - the second node to compare.
    ///
    /// # Returns
    ///
    /// * `Ordering::Less` if `node1` should be processed before `node2`,
    /// * `Ordering::Greater` if `node2` should be processed before `node1`,
    /// * `Ordering::Equal` if both nodes are considered equally good.
    fn comp(&mut self, node1: Node, node2: Node) -> Ordering;
}

/// A wrapper struct for the internal ffi::SCIP_NODESEL
pub struct SCIPNodesel {
    pub(crate) raw: *mut ffi::SCIP_NODESEL,
}

impl SCIPNodesel {
    /// Returns the internal raw pointer of the node selector.
    pub fn inner(&self) -> *mut ffi::SCIP_NODESEL {
        self.raw
    }

    /// Returns the name of the node selector.
    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = ffi::SCIPnodeselGetName(self.raw);
            let name = std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap();
            name.to_string()
        }
    }

    /// Returns the description of the node selector.
    pub fn desc(&self) -> String {
        unsafe {
            let desc_ptr = ffi::SCIPnodeselGetDesc(self.raw);
            let desc = std::ffi::CStr::from_ptr(desc_ptr).to_str().unwrap();
            desc.to_string()
        }
    }

    /// Returns the standard priority of the node selector.
    pub fn std_priority(&self) -> i32 {
        unsafe { ffi::SCIPnodeselGetStdPriority(self.raw) }
    }

    /// Returns the memory saving priority of the node selector.
    pub fn mem_save_priority(&self) -> i32 {
        unsafe { ffi::SCIPnodeselGetMemsavePriority(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::nodesel;
    use crate::{Model, status::Status};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A depth-first-search node selector implemented in Rust.
    struct DfsNodeSel {
        select_calls: Rc<RefCell<usize>>,
    }

    impl NodeSel for DfsNodeSel {
        fn select(&mut self, model: Model<Solving>) -> Option<Node> {
            *self.select_calls.borrow_mut() += 1;
            // Prefer diving into a child, then a sibling, then the best leaf.
            model
                .prio_child()
                .or_else(|| model.prio_sibling())
                .or_else(|| model.best_leaf())
        }

        fn comp(&mut self, node1: Node, node2: Node) -> Ordering {
            // Deeper nodes first (depth-first search).
            node2.depth().cmp(&node1.depth())
        }
    }

    #[test]
    fn dfs_node_selector() {
        let select_calls = Rc::new(RefCell::new(0));
        let ns = DfsNodeSel {
            select_calls: select_calls.clone(),
        };

        // Use a node limit so the selector is exercised on many nodes without
        // having to solve the instance to optimality (which is slow).
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 200)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        model.add(
            nodesel(ns)
                .name("DfsNodeSel")
                .desc("A depth-first-search node selector"),
        );

        let solved = model.solve();
        assert!(matches!(
            solved.status(),
            Status::Optimal | Status::NodeLimit
        ));
        // SCIP consults the node selector once for every node it focuses, plus a
        // final call when no node is left to select.
        assert_eq!(*select_calls.borrow(), solved.n_nodes() + 1);
    }

    #[test]
    fn find_nodesel() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.mps")
            .unwrap();

        // A default SCIP node selector should be found.
        assert!(model.find_nodesel("bfs").is_some());
        // A non-existing node selector should not be found.
        assert!(model.find_nodesel("does-not-exist").is_none());
        // Our custom node selector is not included yet.
        assert!(model.find_nodesel("CustomNodeSel").is_none());

        model.add(
            nodesel(InternalSCIPNodeSelTester)
                .name("CustomNodeSel")
                .desc("A custom node selector"),
        );

        // After inclusion it should be found, with the high default priorities.
        let found = model
            .find_nodesel("CustomNodeSel")
            .expect("custom node selector should be found after inclusion");
        assert_eq!(found.name(), "CustomNodeSel");
        assert_eq!(found.desc(), "A custom node selector");
        assert_eq!(found.std_priority(), 1000000);
        assert_eq!(found.mem_save_priority(), 1000000);
    }

    struct InternalSCIPNodeSelTester;

    impl NodeSel for InternalSCIPNodeSelTester {
        fn select(&mut self, model: Model<Solving>) -> Option<Node> {
            model.best_node()
        }

        fn comp(&mut self, node1: Node, node2: Node) -> Ordering {
            node1
                .lower_bound()
                .partial_cmp(&node2.lower_bound())
                .unwrap_or(Ordering::Equal)
        }
    }
}
