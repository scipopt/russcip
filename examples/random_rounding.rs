use rand::{Rng, SeedableRng};
use russcip::{prelude::*, ParamSetting};
use russcip::{HeurResult, HeurTiming, Heuristic, Model, Solving, VarType};

/// A primal heuristic that performs random rounding at LP solutions
struct RandomRoundingHeur;

impl Heuristic for RandomRoundingHeur {
    fn execute(
        &mut self,
        model: Model<Solving>,
        _timing: HeurTiming,
        node_inf: bool,
    ) -> HeurResult {
        // Skip if the node is infeasible
        if node_inf {
            return HeurResult::DidNotRun;
        }

        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        // Create a new solution
        let sol = model.create_sol();
        let vars = model.vars();

        // Get current LP solution values
        let mut has_fractional = false;
        for var in &vars {
            let lp_val = model.current_val(var);

            // For integer variables, randomly round the fractional values
            if var.var_type() == VarType::Integer || var.var_type() == VarType::Binary {
                let frac_part = lp_val.fract();
                if frac_part > 1e-6 && frac_part < 1.0 - 1e-6 {
                    has_fractional = true;
                    // Randomly round up with probability equal to fractional part
                    let rounded = if rng.gen::<f64>() < frac_part {
                        lp_val.ceil()
                    } else {
                        lp_val.floor()
                    };
                    sol.set_val(var, rounded);
                } else {
                    sol.set_val(var, lp_val.round());
                }
            } else {
                // Keep continuous variables as they are
                sol.set_val(var, lp_val);
            }
        }

        // Only try to add the solution if we actually rounded something
        if !has_fractional {
            return HeurResult::DidNotRun;
        }

        print!("-- RandomRoundingHeur: found a solution: ");
        for var in &vars {
            print!("{} = {}, ", var.name(), sol.val(var));
        }
        println!();

        let sol_val = sol.obj_val();

        // Try to add the rounded solution
        match model.add_sol(sol) {
            Ok(_) => {
                println!(
                    "-- RandomRoundingHeur: Added solution to the model with val {}.",
                    sol_val
                );
                HeurResult::FoundSol
            }
            Err(_) => {
                println!("-- RandomRoundingHeur: Failed to add solution to the model.");
                HeurResult::NoSolFound
            }
        }
    }
}

fn main() {
    // Create a simple MIP model
    let mut model = Model::new()
        .include_default_plugins()
        .read_prob("data/test/simple.mps")
        .unwrap()
        .set_presolving(ParamSetting::Off)
        .set_heuristics(ParamSetting::Off)
        .set_separating(ParamSetting::Off);

    // Add our random rounding heuristic
    model.add(
        heur(RandomRoundingHeur)
            .name("random_round")
            .desc("Random rounding at LP solutions")
            .priority(1000)
            .freq(1)
            .timing(HeurTiming::DURING_LP_LOOP),
    );

    // Solve the model
    let solved_model = model.solve();

    assert!(
        solved_model.n_sols() >= 2,
        "Expected at least 2 solutions, the primal heuristic solution and the optimal solution."
    );
}



mod tests {
    #[test]
    fn test_most_infeasible_branching() {
        main();
    }
}