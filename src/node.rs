use crate::ffi;
use crate::scip::ScipPtr;
use std::rc::Rc;

/// A node in the branch-and-bound tree.
#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) raw: *mut ffi::SCIP_NODE,
    pub(crate) scip: Rc<ScipPtr>,
}

impl Node {
    /// Returns a raw pointer to the underlying `ffi::SCIP_NODE` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_NODE {
        self.raw
    }

    /// Returns the number of the node.
    pub fn number(&self) -> usize {
        unsafe { ffi::SCIPnodeGetNumber(self.raw) as usize }
    }

    /// Returns the depth of the node in the branch-and-bound tree.
    pub fn depth(&self) -> usize {
        unsafe { ffi::SCIPnodeGetDepth(self.raw) as usize }
    }

    /// Returns the lower bound of the node.
    pub fn lower_bound(&self) -> f64 {
        unsafe { ffi::SCIPnodeGetLowerbound(self.raw) }
    }

    /// Returns the parent of the node and `None` if the node is the root node.
    pub fn parent(&self) -> Option<Node> {
        let parent = unsafe { ffi::SCIPnodeGetParent(self.raw) };
        if parent.is_null() {
            None
        } else {
            Some(Node {
                raw: parent,
                scip: self.scip.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::branchrule;
    use crate::{
        SCIPBranchRule, Solving,
        branchrule::{BranchRule, BranchingResult},
        model::Model,
    };

    struct NodeDataBranchRule;

    impl BranchRule for NodeDataBranchRule {
        fn execute(
            &mut self,
            model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            candidates: Vec<crate::branchrule::BranchingCandidate>,
        ) -> BranchingResult {
            let node = model.focus_node();
            if node.number() == 1 {
                assert_eq!(node.depth(), 0);
                assert!(node.lower_bound() < 6777.0);
                assert!(node.parent().is_none());
            } else {
                assert!(node.depth() > 0);
                assert!(node.lower_bound() <= 6777.0);
                assert!(node.parent().is_some());
            }
            BranchingResult::BranchOn(candidates[0].clone())
        }
    }

    #[test]
    fn node_after_solving() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 3)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = NodeDataBranchRule;
        model.add(branchrule(br));
        model.solve();
    }
}
