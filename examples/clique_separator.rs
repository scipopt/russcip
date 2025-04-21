use russcip::prelude::*;
use russcip::{Model, ParamSetting, SCIPSeparator, SeparationResult, Separator, Solving, Variable};
use std::collections::{HashMap, HashSet};
use std::vec;

/// A separator that identifies cliques in the conflict graph of a set partitioning problem
/// and adds corresponding clique cuts.
struct CliqueSeparator {
    /// Threshold for considering a variable as "fractional" (LP value > threshold)
    frac_threshold: f64,
}

impl CliqueSeparator {
    /// Create a new CliqueSeparator with the given threshold
    fn new(frac_threshold: f64) -> Self {
        CliqueSeparator { frac_threshold }
    }

    /// Build a conflict graph for the variables in the model
    /// In a set partitioning problem, two variables conflict if they share a row with coefficient 1
    fn build_conflict_graph(&self, model: &Model<Solving>) -> HashMap<usize, HashSet<usize>> {
        let vars = model.vars();
        let mut conflict_graph: HashMap<usize, HashSet<usize>> = HashMap::new();

        // Initialize the graph with all variables
        for (i, _) in vars.iter().enumerate() {
            conflict_graph.insert(i, HashSet::new());
        }

        // Get constraints and their coefficients
        let mut partitioning_constraints = Vec::new();
        for cons in model.conss() {
            let row = cons.row().unwrap();
            // Only consider equality constraints (partitioning constraints)
            if row.lhs() != 1.0 || row.rhs() != 1.0 {
                continue;
            }
            let vars_in_cons = row
                .cols()
                .iter()
                .map(|col| col.var().index())
                .collect::<Vec<_>>();
            partitioning_constraints.push(vars_in_cons);
        }

        // Build conflicts based on constraints
        for vars_in_cons in partitioning_constraints {
            // Add conflicts between all pairs of variables in this constraint
            for i in 0..vars_in_cons.len() {
                for j in (i + 1)..vars_in_cons.len() {
                    let var_i = vars_in_cons[i];
                    let var_j = vars_in_cons[j];
                    conflict_graph.entry(var_i).or_default().insert(var_j);
                    conflict_graph.entry(var_j).or_default().insert(var_i);
                }
            }
        }

        conflict_graph
    }

    /// Find a clique containing fractional variables in the conflict graph
    /// Uses a simple greedy algorithm
    fn find_clique(
        &self,
        conflict_graph: &HashMap<usize, HashSet<usize>>,
        vars: &[Variable],
    ) -> Vec<usize> {
        // Start with the fractional variable with highest LP value
        let mut clique = vec![0];

        // Greedily add more variables to the clique
        for i in 1..vars.len() {
            let var_idx = vars[i].index();

            // Check if this variable conflicts with all variables in the current clique
            let mut can_add = true;
            for &clique_var in &clique {
                let clique_var_idx = vars[clique_var].index();
                if !conflict_graph[&var_idx].contains(&clique_var_idx) {
                    can_add = false;
                    break;
                }
            }

            if can_add {
                clique.push(i);
            }
        }

        clique
    }
}

impl Separator for CliqueSeparator {
    fn execute_lp(&mut self, mut model: Model<Solving>, sepa: SCIPSeparator) -> SeparationResult {
        println!("-- CliqueSeparator: Executing LP separation");

        let vars = model.vars();

        // Get current LP values
        let lp_values: Vec<f64> = vars.iter().map(|var| model.current_val(var)).collect();

        // Check if we have any fractional variables
        let has_fractional = lp_values
            .iter()
            .any(|&val| val > self.frac_threshold && val < 1.0 - self.frac_threshold);

        if !has_fractional {
            println!("-- CliqueSeparator: No fractional variables found");
            return SeparationResult::DidNotFind;
        }

        // Build the conflict graph
        let conflict_graph = self.build_conflict_graph(&model);

        println!(
            "-- CliqueSeparator: Conflict graph built with {} nodes",
            conflict_graph.len()
        );

        // Find a clique with fractional variables
        let clique = self.find_clique(&conflict_graph, &vars);
        println!(
            "-- CliqueSeparator: Found clique with {} variables",
            clique.len()
        );

        if clique.len() <= 1 {
            return SeparationResult::DidNotFind;
        }

        // Create a clique cut: sum of variables in the clique <= 1
        let mut clique_cut = sepa
            .create_empty_row(
                &model,
                "clique_cut",
                -f64::INFINITY,
                1.0,
                false,
                false,
                false,
            )
            .unwrap();

        // Calculate sum of LP values for variables in the clique
        let sum_lp_values = clique.iter().map(|&i| lp_values[i]).sum::<f64>();

        // Add variables to the clique cut
        for &var_idx in &clique {
            clique_cut.set_coeff(&vars[var_idx], 1.0);
        }

        // Only add the cut if it's violated
        if sum_lp_values > 1.0 + 1e-6 {
            println!(
                "-- CliqueSeparator: Found violated clique cut with {} variables, sum of LP values: {}",
                clique.len(),
                sum_lp_values
            );

            model.add_cut(clique_cut, true);
            SeparationResult::Separated
        } else {
            SeparationResult::DidNotFind
        }
    }
}

fn main() {
    // Create a simple set partitioning problem
    let mut model = Model::new()
        .include_default_plugins()
        .create_prob("setpart_clique")
        .set_presolving(ParamSetting::Off)
        .set_separating(ParamSetting::Off)
        .set_heuristics(ParamSetting::Off)
        .set_param("branching/pscost/priority", 1000000)
        .set_param("misc/usesymmetry", 0);

    let x1 = model.add(var().bin().obj(1.0));
    let x2 = model.add(var().bin().obj(1.0));
    let x3 = model.add(var().bin().obj(1.0));
    let x4 = model.add(var().bin().obj(1.0));
    let x5 = model.add(var().bin().obj(1.0));
    let x6 = model.add(var().bin().obj(1.0));

    // Add set partitioning constraints
    model.add_cons_set_part(vec![&x1, &x2, &x4], "set1");
    model.add_cons_set_part(vec![&x1, &x3, &x5], "set2");
    model.add_cons_set_part(vec![&x2, &x3, &x6], "set3");

    // Add our clique separator
    model.add(
        sepa(CliqueSeparator::new(0.1))
            .name("clique_separator")
            .desc("Clique separator for set partitioning problems"),
    );

    // Solve the model
    let solved_model = model.solve();

    // Print results
    println!("\nSolution status: {:?}", solved_model.status());
    println!("Objective value: {:.2}", solved_model.obj_val());

    assert_eq!(solved_model.status(), Status::Optimal);
    assert_eq!(solved_model.n_nodes(), 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clique_separator() {
        main();
    }
}
