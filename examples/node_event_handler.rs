use std::cell::RefCell;
use std::rc::Rc;

use russcip::prelude::*;
use russcip::{Event, EventMask, Eventhdlr, Model, SCIPEventhdlr, Solving};

#[derive(Debug, Default)]
struct NodeInfoEventHandler {
    runs: Rc<RefCell<usize>>,
}
impl NodeInfoEventHandler {
    fn new(runs: Rc<RefCell<usize>>) -> Self {
        Self { runs }
    }
}

impl Eventhdlr for NodeInfoEventHandler {
    fn get_type(&self) -> EventMask {
        // Listen for node events (when nodes are focused, solved, etc.)
        EventMask::NODE_FOCUSED
    }

    fn execute(&mut self, model: Model<Solving>, _eventhdlr: SCIPEventhdlr, _event: Event) {
        let current_node = model.focus_node();
        *self.runs.borrow_mut() += 1;

        println!(
            "-- NodeInfoEventHandler: at Node {}: depth = {}, parent = {}",
            current_node.number(),
            current_node.depth(),
            current_node
                .parent()
                .map_or("none".to_string(), |p| p.number().to_string())
        );
    }
}

fn main() {
    // Create a new model and create the problem
    let mut model = Model::new()
        .hide_output()
        .include_default_plugins()
        .read_prob("data/test/simple.mps")
        .unwrap()
        .set_heuristics(ParamSetting::Off)
        .set_separating(ParamSetting::Off)
        .set_presolving(ParamSetting::Off)
        .set_param("branching/pscost/priority", 1000000);

    let seen_nodes = Rc::new(RefCell::new(0));
    model.add(eventhdlr(NodeInfoEventHandler::new(seen_nodes.clone())).name("NodeInfoPrinter"));

    let solved = model.solve();
    assert!(*seen_nodes.borrow() > 0);
    assert_eq!(solved.n_nodes(), *seen_nodes.borrow());
    println!(
        "-- NodeInfoEventHandler: Total nodes seen = {}",
        *seen_nodes.borrow()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_event_handler() {
        main();
    }
}
