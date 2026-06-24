use crate::builder::CanBeAddedToModel;
use crate::{Model, NodeSel, ProblemCreated};

/// A builder for easily creating node selectors. It can be created using the `nodesel` function.
pub struct NodeSelBuilder<N: NodeSel> {
    name: Option<String>,
    desc: Option<String>,
    std_priority: i32,
    mem_save_priority: i32,
    nodesel: N,
}

impl<N: NodeSel> NodeSelBuilder<N> {
    /// Creates a new `NodeSelBuilder` with the given node selector.
    ///
    /// Defaults:
    /// - `name`: empty string
    /// - `desc`: empty string
    /// - `std_priority`: 1000000 (higher than all default SCIP node selectors, so the custom one is used)
    /// - `mem_save_priority`: 1000000
    pub fn new(nodesel: N) -> Self {
        NodeSelBuilder {
            name: None,
            desc: None,
            std_priority: 1000000,
            mem_save_priority: 1000000,
            nodesel,
        }
    }

    /// Sets the name of the node selector.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the description of the node selector.
    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    /// Sets the standard priority of the node selector.
    ///
    /// Among all included node selectors, the one with the highest standard
    /// priority is used in standard mode. A higher value indicates a higher
    /// priority.
    pub fn std_priority(mut self, priority: i32) -> Self {
        self.std_priority = priority;
        self
    }

    /// Sets the memory saving priority of the node selector.
    ///
    /// When SCIP switches to memory saving mode, the node selector with the
    /// highest memory saving priority is used instead. A higher value indicates
    /// a higher priority.
    pub fn mem_save_priority(mut self, priority: i32) -> Self {
        self.mem_save_priority = priority;
        self
    }
}

/// Creates a new default `NodeSelBuilder` from a node selector.
/// This function allows you to write:
/// ```rust
/// use russcip::{NodeSel, Node, Solving};
/// use russcip::prelude::*;
/// use std::cmp::Ordering;
///
/// struct MyNodeSel;
/// impl NodeSel for MyNodeSel {
///     fn select(&mut self, model: Model<Solving>) -> Option<Node> {
///         model.best_node()
///     }
///     fn comp(&mut self, node1: Node, node2: Node) -> Ordering {
///         node1.lower_bound().partial_cmp(&node2.lower_bound()).unwrap_or(Ordering::Equal)
///     }
/// }
///
/// let nsel = nodesel(MyNodeSel)
///     .name("My Node Selector")
///     .desc("A custom node selector");
///
/// let mut model = Model::default();
/// model.add(nsel);
/// ```
pub fn nodesel<N: NodeSel>(nodesel: N) -> NodeSelBuilder<N> {
    NodeSelBuilder::new(nodesel)
}

impl<N: NodeSel + 'static> CanBeAddedToModel<ProblemCreated> for NodeSelBuilder<N> {
    type Return = ();

    fn add(self, model: &mut Model<ProblemCreated>) {
        // Use empty strings as defaults if name or description are not provided.
        let name = self.name.unwrap_or_else(|| "".into());
        let desc = self.desc.unwrap_or_else(|| "".into());
        let nodesel_box = Box::new(self.nodesel);
        model.include_nodesel(
            &name,
            &desc,
            self.std_priority,
            self.mem_save_priority,
            nodesel_box,
        );
    }
}

impl<N: NodeSel> From<N> for NodeSelBuilder<N> {
    fn from(nodesel: N) -> Self {
        NodeSelBuilder::new(nodesel)
    }
}
