use petgraph::prelude::*;
use russcip::prelude::*;
use russcip::{Conshdlr, ConshdlrResult, SCIPConshdlr, Solution, Solving, Variable};
use std::collections::HashMap;

/// Find all "subtours" in an undirected graph. A subtour corresponds to a connected component.
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

            subtours.push(component_nodes);
        }
    }

    subtours
}

type TspGraph = UnGraph<(), f64>;
type VarMap = HashMap<EdgeIndex, Variable>;

fn solution_to_graph(g: &TspGraph, vars: &VarMap, sol: &Solution) -> UnGraph<(), ()> {
    UnGraph::from_edges(
        vars.iter()
            .filter(|(_, var)| sol.val(var) > 0.5)
            .map(|(edge, _)| g.edge_endpoints(*edge).unwrap()),
    )
}

/// A constraint handler that enforces the TSP subtour elimination constraint.
struct SubtourConshdlr {
    vars: VarMap,
    graph: TspGraph,
}

impl Conshdlr for SubtourConshdlr {
    fn check(
        &mut self,
        _model: Model<Solving>,
        _conshdlr: SCIPConshdlr,
        solution: &Solution,
    ) -> bool {
        let subtours = find_subtours(&solution_to_graph(&self.graph, &self.vars, solution));
        subtours.len() == 1
    }

    fn enforce(&mut self, mut model: Model<Solving>, _conshdlr: SCIPConshdlr) -> ConshdlrResult {
        let edges_in_lp_sol = self
            .vars
            .iter()
            .filter(|v| model.current_val(v.1) > 0.5)
            .collect::<Vec<_>>();

        let sol_graph = UnGraph::from_edges(
            edges_in_lp_sol
                .iter()
                .map(|(edge, _)| self.graph.edge_endpoints(**edge).unwrap()),
        );

        let subtours = find_subtours(&sol_graph);

        if subtours.len() == 1 {
            return ConshdlrResult::Feasible;
        }

        // Add a constraint to eliminate each subtour
        for subtour in &subtours {
            model.add(
                cons().le(subtour.len() as f64 - 1.0).expr(
                    self.graph
                        .edge_indices()
                        .filter(|edge| {
                            let (source, target) = self.graph.edge_endpoints(*edge).unwrap();
                            subtour.contains(&source) && subtour.contains(&target)
                        })
                        .map(|edge| (self.vars.get(&edge).unwrap(), 1.0)),
                ),
            );
        }
        ConshdlrResult::ConsAdded
    }
}

struct TspResult {
    tour: Vec<NodeIndex>,
    cost: f64,
}

/// Solve the TSP problem and return the edges in the solution.
fn solve_tsp(graph: TspGraph) -> Result<TspResult, Status> {
    let mut model = Model::default(); //minimal_model();
    let mut vars = HashMap::new();

    for edge in graph.edge_indices() {
        let (source_id, target_id) = graph.edge_endpoints(edge).unwrap();
        let (source, target) = (source_id.index(), target_id.index());
        let name = format!("x_{}_{}", source, target);
        let distance = *graph.edge_weight(edge).unwrap();
        let var = model.add(var().bin().obj(distance).name(&name));
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

    let conshdlr = SubtourConshdlr {
        vars: vars.clone(),
        graph: graph.clone(),
    };
    model.include_conshdlr(
        "SEC",
        "Subtour Elimination Constraint",
        -1,
        -1,
        Box::new(conshdlr),
    );

    let solved = model.solve();
    assert_eq!(solved.status(), Status::Optimal);
    match solved.status() {
        Status::Optimal => {
            let sol = solved.best_sol().unwrap();
            let tour = find_subtours(&solution_to_graph(&graph, &vars, &sol))
                .pop()
                .unwrap();
            Ok(TspResult {
                tour,
                cost: sol.obj_val(),
            })
        }
        _ => Err(solved.status()),
    }
}

fn main() {
    let mut edges = vec![];
    POINTS.iter().enumerate().for_each(|(i, [x1, y1])| {
        POINTS
            .iter()
            .enumerate()
            .skip(i + 1)
            .for_each(|(j, [x2, y2])| {
                let distance = ((x1 - x2) as f64).hypot((y1 - y2) as f64);
                edges.push((i as u32, j as u32, distance));
            });
    });

    let g: TspGraph = Graph::from_edges(edges);
    match solve_tsp(g) {
        Ok(result) => {
            println!(
                "found tour on {} nodes with cost {}",
                result.tour.len(),
                result.cost
            );
            assert_eq!(result.tour.len(), POINTS.len());
        }
        Err(status) => {
            println!("failed to solve: {:?}", status);
        }
    }
}

/// Instance a280.tsp from TSPLIB95
/// Reinelt, G. (1995). Tsplib95. Interdisziplinäres Zentrum für Wissenschaftliches Rechnen (IWR), Heidelberg, 338, 1-16.
static POINTS: &[[i32; 2]] = &[
    [288, 149],
    [288, 129],
    [270, 133],
    [256, 141],
    [256, 157],
    [246, 157],
    [236, 169],
    [228, 169],
    [228, 161],
    [220, 169],
    [212, 169],
    [204, 169],
    [196, 169],
    [188, 169],
    [196, 161],
    [188, 145],
    [172, 145],
    [164, 145],
    [156, 145],
    [148, 145],
    [140, 145],
    [148, 169],
    [164, 169],
    [172, 169],
    [156, 169],
    [140, 169],
    [132, 169],
    [124, 169],
    [116, 161],
    [104, 153],
    [104, 161],
    [104, 169],
    [90, 165],
    [80, 157],
    [64, 157],
    [64, 165],
    [56, 169],
    [56, 161],
    [56, 153],
    [56, 145],
    [56, 137],
    [56, 129],
    [56, 121],
    [40, 121],
    [40, 129],
    [40, 137],
    [40, 145],
    [40, 153],
    [40, 161],
    [40, 169],
    [32, 169],
    [32, 161],
    [32, 153],
    [32, 145],
    [32, 137],
    [32, 129],
    [32, 121],
    [32, 113],
    [40, 113],
    [56, 113],
    [56, 105],
    [48, 99],
    [40, 99],
    [32, 97],
    [32, 89],
    [24, 89],
    [16, 97],
    [16, 109],
    [8, 109],
    [8, 97],
    [8, 89],
    [8, 81],
    [8, 73],
    [8, 65],
    [8, 57],
    [16, 57],
    [8, 49],
    [8, 41],
    [24, 45],
    [32, 41],
    [32, 49],
    [32, 57],
    [32, 65],
    [32, 73],
    [32, 81],
    [40, 83],
    [40, 73],
    [40, 63],
    [40, 51],
    [44, 43],
    [44, 35],
    [44, 27],
    [32, 25],
    [24, 25],
    [16, 25],
    [16, 17],
    [24, 17],
    [32, 17],
    [44, 11],
    [56, 9],
    [56, 17],
    [56, 25],
    [56, 33],
    [56, 41],
    [64, 41],
    [72, 41],
    [72, 49],
    [56, 49],
    [48, 51],
    [56, 57],
    [56, 65],
    [48, 63],
    [48, 73],
    [56, 73],
    [56, 81],
    [48, 83],
    [56, 89],
    [56, 97],
    [104, 97],
    [104, 105],
    [104, 113],
    [104, 121],
    [104, 129],
    [104, 137],
    [104, 145],
    [116, 145],
    [124, 145],
    [132, 145],
    [132, 137],
    [140, 137],
    [148, 137],
    [156, 137],
    [164, 137],
    [172, 125],
    [172, 117],
    [172, 109],
    [172, 101],
    [172, 93],
    [172, 85],
    [180, 85],
    [180, 77],
    [180, 69],
    [180, 61],
    [180, 53],
    [172, 53],
    [172, 61],
    [172, 69],
    [172, 77],
    [164, 81],
    [148, 85],
    [124, 85],
    [124, 93],
    [124, 109],
    [124, 125],
    [124, 117],
    [124, 101],
    [104, 89],
    [104, 81],
    [104, 73],
    [104, 65],
    [104, 49],
    [104, 41],
    [104, 33],
    [104, 25],
    [104, 17],
    [92, 9],
    [80, 9],
    [72, 9],
    [64, 21],
    [72, 25],
    [80, 25],
    [80, 25],
    [80, 41],
    [88, 49],
    [104, 57],
    [124, 69],
    [124, 77],
    [132, 81],
    [140, 65],
    [132, 61],
    [124, 61],
    [124, 53],
    [124, 45],
    [124, 37],
    [124, 29],
    [132, 21],
    [124, 21],
    [120, 9],
    [128, 9],
    [136, 9],
    [148, 9],
    [162, 9],
    [156, 25],
    [172, 21],
    [180, 21],
    [180, 29],
    [172, 29],
    [172, 37],
    [172, 45],
    [180, 45],
    [180, 37],
    [188, 41],
    [196, 49],
    [204, 57],
    [212, 65],
    [220, 73],
    [228, 69],
    [228, 77],
    [236, 77],
    [236, 69],
    [236, 61],
    [228, 61],
    [228, 53],
    [236, 53],
    [236, 45],
    [228, 45],
    [228, 37],
    [236, 37],
    [236, 29],
    [228, 29],
    [228, 21],
    [236, 21],
    [252, 21],
    [260, 29],
    [260, 37],
    [260, 45],
    [260, 53],
    [260, 61],
    [260, 69],
    [260, 77],
    [276, 77],
    [276, 69],
    [276, 61],
    [276, 53],
    [284, 53],
    [284, 61],
    [284, 69],
    [284, 77],
    [284, 85],
    [284, 93],
    [284, 101],
    [288, 109],
    [280, 109],
    [276, 101],
    [276, 93],
    [276, 85],
    [268, 97],
    [260, 109],
    [252, 101],
    [260, 93],
    [260, 85],
    [236, 85],
    [228, 85],
    [228, 93],
    [236, 93],
    [236, 101],
    [228, 101],
    [228, 109],
    [228, 117],
    [228, 125],
    [220, 125],
    [212, 117],
    [204, 109],
    [196, 101],
    [188, 93],
    [180, 93],
    [180, 101],
    [180, 109],
    [180, 117],
    [180, 125],
    [196, 145],
    [204, 145],
    [212, 145],
    [220, 145],
    [228, 145],
    [236, 145],
    [246, 141],
    [252, 125],
    [260, 129],
    [280, 133],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsp_a280() {
        main();
    }
}
