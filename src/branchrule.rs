use crate::{ffi, Model, Solving};
use scip_sys::SCIP_Result;

/// A trait for defining custom branching rules.
pub trait BranchRule {
    /// Executes the branching rule on the given candidates and returns the result.
    fn execute(
        &mut self,
        candidates: Vec<BranchingCandidate>,
        model: Model<Solving>,
    ) -> BranchingResult;
}

/// The result of a branching rule execution.
#[derive(Debug, Clone, PartialEq)]
pub enum BranchingResult {
    /// The branching rule did not run
    DidNotRun,
    /// Initiate branching on the given candidate
    BranchOn(BranchingCandidate),
    /// Current node is detected to be infeasible and can be cut off
    CutOff,
    /// A custom branching scheme is implemented
    CustomBranching,
    /// A cutting plane is added
    Separated,
    /// Reduced the domain of a variable such that the current LP solution becomes infeasible
    ReduceDom,
    /// A constraint was added
    ConsAdded,
}

impl From<BranchingResult> for SCIP_Result {
    fn from(val: BranchingResult) -> Self {
        match val {
            BranchingResult::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            BranchingResult::BranchOn(_) => ffi::SCIP_Result_SCIP_BRANCHED,
            BranchingResult::CutOff => ffi::SCIP_Result_SCIP_CUTOFF,
            BranchingResult::CustomBranching => ffi::SCIP_Result_SCIP_BRANCHED,
            BranchingResult::Separated => ffi::SCIP_Result_SCIP_SEPARATED,
            BranchingResult::ReduceDom => ffi::SCIP_Result_SCIP_REDUCEDDOM,
            BranchingResult::ConsAdded => ffi::SCIP_Result_SCIP_CONSADDED,
        }
    }
}

/// A candidate for branching.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchingCandidate {
    /// The index of the variable to branch on in the current subproblem.
    pub var_prob_id: usize,
    /// The LP solution value of the variable.
    pub lp_sol_val: f64,
    /// The fractional part of the LP solution value of the variable.
    pub frac: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelWithProblem;
    use crate::{model::Model, status::Status, Solving};

    struct FirstChoosingBranchingRule {
        pub chosen: Option<BranchingCandidate>,
    }

    impl BranchRule for FirstChoosingBranchingRule {
        fn execute(
            &mut self,
            candidates: Vec<BranchingCandidate>,
            model: Model<Solving>,
        ) -> BranchingResult {
            self.chosen = Some(candidates[0].clone());
            BranchingResult::DidNotRun
        }
    }

    #[test]
    fn choosing_first_branching_rule() {
        let br = FirstChoosingBranchingRule { chosen: None };

        let model = Model::new()
            .set_longint_param("limits/nodes", 2) // only call brancher once
            .unwrap()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap()
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br));

        let solved = model.solve();
        assert_eq!(solved.status(), Status::NodeLimit);
        // assert!(br.chosen.is_some());
        // let candidate = br.chosen.unwrap();
        // assert!(candidate.lp_sol_val.fract() > 0.);
        // assert!(candidate.frac > 0. && candidate.frac < 1.);
    }

    struct CuttingOffBranchingRule;

    impl BranchRule for CuttingOffBranchingRule {
        fn execute(
            &mut self,
            _candidates: Vec<BranchingCandidate>,
            model: Model<Solving>,
        ) -> BranchingResult {
            BranchingResult::CutOff
        }
    }

    #[test]
    fn cutting_off_branching_rule() {
        let br = CuttingOffBranchingRule {};

        // create model from miplib instance gen-ip054
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap()
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();
        assert_eq!(model.n_nodes(), 1);
    }

    struct FirstBranchingRule;

    impl BranchRule for FirstBranchingRule {
        fn execute(
            &mut self,
            candidates: Vec<BranchingCandidate>,
            model: Model<Solving>,
        ) -> BranchingResult {
            assert!(model.n_vars() >= candidates.len());
            BranchingResult::BranchOn(candidates[0].clone())
        }
    }

    #[test]
    fn first_branching_rule() {
        let model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = FirstBranchingRule;
        let solved = model
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();

        assert!(solved.n_nodes() > 1);
    }

    struct CustomBranchingRule;

    impl BranchRule for CustomBranchingRule {
        fn execute(
            &mut self,
            _candidates: Vec<BranchingCandidate>,
            mut model: Model<Solving>,
        ) -> BranchingResult {
            model.create_child();
            BranchingResult::CustomBranching
        }
    }

    #[test]
    fn custom_branching_rule() {
        let model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = CustomBranchingRule;
        let solved = model
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();

        assert!(solved.n_nodes() > 1);
    }

    struct HighestBoundBranchRule;

    impl BranchRule for HighestBoundBranchRule {
        fn execute(
            &mut self,
            candidates: Vec<BranchingCandidate>,
            model: Model<Solving>,
        ) -> BranchingResult {
            let mut max_bound = f64::NEG_INFINITY;
            let mut max_candidate = None;
            for candidate in candidates {
                let var = model.var_in_prob(candidate.var_prob_id).unwrap();
                let bound = var.ub();
                if bound > max_bound {
                    max_bound = bound;
                    max_candidate = Some(candidate);
                }
            }

            if let Some(candidate) = max_candidate {
                BranchingResult::BranchOn(candidate)
            } else {
                BranchingResult::DidNotRun
            }
        }
    }

    #[test]
    fn highest_bound_branch_rule() {
        let model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = HighestBoundBranchRule;
        let solved = model
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();

        assert!(solved.n_nodes() > 1);
    }
}
