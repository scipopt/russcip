use russcip::{
    BranchRule, BranchingCandidate, BranchingResult, ParamSetting, SCIPBranchRule, Solving,
    prelude::*,
};

/// A branching rule that implements the most infeasible branching strategy.
/// It selects the variable with the highest fractionality (closest to 0.5) to branch on.
struct MostInfeasibleBranching;

impl BranchRule for MostInfeasibleBranching {
    fn execute(
        &mut self,
        model: Model<Solving>,
        _branchrule: SCIPBranchRule,
        candidates: Vec<BranchingCandidate>,
    ) -> BranchingResult {
        // Find the candidate with the highest fractionality
        let mut best_candidate = candidates.first().unwrap().clone();
        let mut best_fractionality = (best_candidate.frac - 0.5).abs();

        for candidate in candidates {
            let fractionality = (candidate.frac - 0.5).abs();
            if fractionality > best_fractionality {
                best_fractionality = fractionality;
                best_candidate = candidate;
            }
        }

        let var = model.var_in_prob(best_candidate.var_prob_id).unwrap();
        println!(
            "-- MostInfeasibleBranching: Branching on variable {} with fractionality {}",
            var.name(),
            best_candidate.frac
        );
        BranchingResult::BranchOn(best_candidate)
    }
}

fn main() {
    // Initialize SCIP
    let mut model = Model::new()
        .include_default_plugins()
        .read_prob("data/test/simple.mps")
        .expect("Failed to read problem file")
        .set_presolving(ParamSetting::Off)
        .set_heuristics(ParamSetting::Off)
        .set_separating(ParamSetting::Off);

    // Add the custom branching rule
    model.add(
        branchrule(MostInfeasibleBranching)
            .name("MostInfeasible")
            .desc("Most infeasible branching rule"),
    );

    let solved = model.solve();

    assert_eq!(solved.status(), Status::Optimal);
    assert_eq!(solved.n_nodes(), 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_most_infeasible_branching() {
        main();
    }
}
