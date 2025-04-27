use russcip::prelude::*;
use russcip::{
    Model, ParamSetting, Row, SCIPSeparator, SeparationResult, Separator, Solving, VarType,
    Variable,
};

/// A separator that generates Gomory Mixed Integer (GMI) cuts
struct GmiSeparator {
    ncuts: usize,
}

impl GmiSeparator {
    fn new() -> Self {
        GmiSeparator { ncuts: 0 }
    }

    /// Generate a GMI cut from an LP row
    fn get_gmi_cut(
        &self,
        model: &Model<Solving>,
        vars: &[Variable],
        row: &Row,
        primsol: f64,
    ) -> Option<(Vec<f64>, f64)> {
        let mut cutcoefs = vec![0.0; vars.len()];

        // Compute cut fractionality f0 and f0/(1-f0)
        let f0 = model.frac(primsol);
        let ratio_f0_compl = f0 / (1.0 - f0);

        // RHS of the cut is the fractional part of the LP solution
        let cutrhs = -f0;

        // Get row values
        let cols = row.cols();

        // Extract coefficients for integer variables
        for col in cols {
            let var = col.var();
            let coef = col.primal_sol();
            let pos = col.index();

            let cutelem = if var.var_type() != VarType::Continuous {
                let frac_part = model.frac(coef);
                if frac_part > f0 {
                    -((1.0 - frac_part) * ratio_f0_compl)
                } else {
                    -frac_part
                }
            } else {
                if coef < 0.0 {
                    coef * ratio_f0_compl
                } else {
                    -coef
                }
            };

            if !model.eq(cutelem, 0.0) {
                cutcoefs[pos] = cutelem;
            }
        }

        println!("-- GMI cut: {:?}", cutcoefs);
        Some((cutcoefs, cutrhs))
    }
}

impl Separator for GmiSeparator {
    fn execute_lp(&mut self, mut model: Model<Solving>, sepa: SCIPSeparator) -> SeparationResult {
        let vars = model.vars();

        let mut result = SeparationResult::DidNotFind;

        // Try all integer variables with fractional values
        for var in vars.iter() {
            if var.var_type() == VarType::Continuous {
                continue;
            }

            let primsol = model.current_val(var);
            let frac = model.frac(primsol);

            // Only try variables with significant fractionality
            if frac <= 0.005 || frac >= 0.995 {
                continue;
            }

            // Get the row corresponding to this basic variable
            if let Some(col) = var.col() {
                let rows = col.rows();
                if let Some(row) = rows.first() {
                    // Generate the GMI cut
                    if let Some((cutcoefs, cutrhs)) = self.get_gmi_cut(&model, &vars, row, primsol)
                    {
                        // Create a new cut
                        let mut cut = sepa
                            .create_empty_row(
                                &model,
                                &format!("gmi{}_x{}", self.ncuts, var.index()),
                                cutrhs,
                                cutrhs,
                                false,
                                false,
                                false,
                            )
                            .unwrap();

                        // Add non-zero coefficients
                        for (j, &coef) in cutcoefs.iter().enumerate() {
                            if !model.eq(coef, 0.0) {
                                cut.set_coeff(&vars[j], coef);
                            }
                        }

                        // Skip empty cuts
                        if cut.n_non_zeroes() == 0 {
                            if cutrhs < 0.0 {
                                return SeparationResult::Cutoff;
                            }
                            continue;
                        }

                        // Add the cut
                        model.add_cut(cut, true);
                        self.ncuts += 1;
                        result = SeparationResult::Separated;
                    }
                }
            }
        }

        result
    }
}

fn main() {
    // Create a simple MIP to test the GMI separator
    let mut model = Model::new()
        .include_default_plugins()
        .read_prob("data/test/gen-ip054.mps")
        .unwrap()
        .set_separating(ParamSetting::Off)
        .set_presolving(ParamSetting::Off)
        .set_heuristics(ParamSetting::Off);

    // Add our GMI separator
    model.add(
        sepa(GmiSeparator::new())
            .name("gmi_separator")
            .desc("Gomory Mixed Integer Cut Separator")
            .priority(100000)
            .freq(1),
    );

    // Solve the model
    let solved_model = model.solve();

    // Print results
    println!("\nSolution status: {:?}", solved_model.status());
    println!("Objective value: {:.2}", solved_model.obj_val());
    println!("Number of nodes: {}", solved_model.n_nodes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmi_separator() {
        main();
    }
}
