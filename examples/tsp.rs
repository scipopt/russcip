use petgraph::prelude::*;
use russcip::prelude::*;
use std::collections::HashMap;
use std::ptr::null_mut;
use petgraph::algo::connected_components;
use russcip::{ffi, minimal_model, Conshdlr, ConshdlrResult, SCIPConshdlr, Solution, Solving, Variable};

/// Find all "subtours" in an undirected graph.
/// A "subtour" here is defined as a connected component with fewer nodes
/// than the entire graph. For TSP, that usually indicates a cycle
/// that doesn't include every node.
fn find_subtours(graph: &UnGraph<(), ()>) -> Vec<Vec<NodeIndex>> {
    let n = graph.node_count();
    let mut visited = vec![false; n];
    let mut subtours = Vec::new();

    // Standard BFS/DFS approach to find connected components
    for start in graph.node_indices() {
        if !visited[start.index()] {
            // We'll collect all nodes in this component in 'component_nodes'
            let mut stack = vec![start];
            let mut component_nodes = Vec::new();
            visited[start.index()] = true;

            // BFS/DFS
            while let Some(u) = stack.pop() {
                component_nodes.push(u);
                for v in graph.neighbors(u) {
                    if !visited[v.index()] {
                        visited[v.index()] = true;
                        stack.push(v);
                    }
                }
            }

            // If the component size is less than the entire graph, call it a "subtour"
            if component_nodes.len() < n {
                subtours.push(component_nodes);
            }
        }
    }

    subtours
}

/// A constraint handler that enforces the TSP subtour elimination constraint.
struct SubtourConshdlr {
    vars: HashMap<EdgeIndex, Variable>,
    graph: UnGraph<(), ()>,
}

impl Conshdlr for SubtourConshdlr {
    fn check(&mut self, _model: Model<Solving>, _conshdlr: SCIPConshdlr, solution: &Solution) -> bool {
        let sol_graph = UnGraph::from_edges(
            self.vars.iter().filter(|(_, var)| solution.val(var) > 0.5).map(|(edge, _)| {
                self.graph.edge_endpoints(*edge).unwrap()
            }),
        );

        let subtours = find_subtours(&sol_graph);
        subtours.is_empty()
    }

    fn enforce(&mut self, mut model: Model<Solving>, _conshdlr: SCIPConshdlr) -> ConshdlrResult {
        let edges_in_lp_sol = self.vars.iter().filter(
            |v| {
                let val = unsafe { ffi::SCIPgetSolVal(model.scip_ptr(), null_mut(), v.1.inner())
                };
                val > 0.5
            }
        ).collect::<Vec<_>>();

        let sol_graph = UnGraph::from_edges(
            edges_in_lp_sol.iter().map(|(edge, _)| {
                self.graph.edge_endpoints(**edge).unwrap()
            }),
        );

        let subtours = find_subtours(&sol_graph);

        if subtours.is_empty() {
            return ConshdlrResult::Feasible;
        }

        // Add a constraint to eliminate each subtour
        for subtour in &subtours {
            let mut c = cons().le(subtour.len() as f64 - 1.0);
            let edge_vars_in_subtour = self
                .graph
                .edge_indices()
                .filter(|edge| {
                    let (source, target) = self.graph.edge_endpoints(*edge).unwrap();
                    subtour.contains(&source) && subtour.contains(&target)
                })
                .map(|edge| self.vars.get(&edge).unwrap());

            for var in edge_vars_in_subtour {
                c = c.coef(var, 1.0);
            }

            model.add(c);
        }
        ConshdlrResult::ConsAdded
    }
}


/// Solve the TSP problem and return the edges in the solution.
fn solve_tsp(graph: UnGraph<(), ()>) -> Vec<(usize, usize)> {
    let mut model = minimal_model();
    let mut vars = HashMap::new();

    for edge in graph.edge_indices() {
        let (source_id, target_id) = graph.edge_endpoints(edge).unwrap();
        let (source, target) = (source_id.index(), target_id.index());
        let name = format!("x_{}_{}", source, target);
        let var = model.add(var().bin().obj(1.0).name(&name));
        vars.insert(edge, var);
    }

    for node in graph.node_indices() {
        let mut c = cons().eq(2.0);
        for edge in graph.edges(node) {
            let var = vars.get(&edge.id()).unwrap();
            if edge.source() == node || edge.target() == node {
                c = c.coef(var, 1.0);
            }
        }
        model.add(c);
    }


    let conshdlr = SubtourConshdlr { vars: vars.clone(), graph: graph.clone() };
    model.include_conshdlr("TSPConshdlr", "TSP constraint handler", -1, -1, Box::new(conshdlr));

    let solved = model.solve();
    assert_eq!(solved.status(), Status::Optimal);

    let sol = solved.best_sol().unwrap();
    let edges_in_solution = vars
        .iter()
        .filter(|(_, var)| sol.val(var) > 0.5)
        .map(|x| {
            let edge_id = x.0;
            let edge = graph.edge_endpoints(*edge_id).unwrap();
            (edge.0.index(), edge.1.index())
        })
        .collect::<Vec<_>>();

    edges_in_solution
}

fn main() {
    let graph_size = 100;
    let mut edges = vec![];
    for i in 0..graph_size {
        for j in i + 1..graph_size {
            edges.push((i, j));
        }
    }
    let g: UnGraph<(), ()> = Graph::from_edges(edges);

    let edges = solve_tsp(g);
    let sol_graph: Graph<usize, (), Undirected, usize> = Graph::from_edges(edges);
    assert_eq!(connected_components(&sol_graph), 1)
}
