use crate::ffi;
use crate::variable::Variable;
use scip_sys::SCIP_Result;
use std::rc::Rc;

/// A trait for defining custom branching rules.
pub trait BranchRule {
    /// Executes the branching rule on the given candidates and returns the result.
    fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> BranchingResult;
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
    /// The variable to branch on.
    pub var: Rc<Variable>,
    /// The LP solution value of the variable.
    pub lp_sol_val: f64,
    /// The fractional part of the LP solution value of the variable.
    pub frac: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelWithProblem;
    use crate::Solving;
    use crate::{model::Model, status::Status};

    struct PanickingBranchingRule;
    impl BranchRule for PanickingBranchingRule {
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
            panic!("Not implemented")
        }
    }

    #[test]
    #[should_panic]
    fn panicking_branchrule() {
        let br = PanickingBranchingRule {};

        Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap()
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();
    }

    struct FirstChoosingBranchingRule {
        pub chosen: Option<BranchingCandidate>,
    }

    impl BranchRule for FirstChoosingBranchingRule {
        fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> BranchingResult {
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
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
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

    struct FirstBranchingRule {
        model: Model<Solving>,
    }

    impl BranchRule for FirstBranchingRule {
        fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> BranchingResult {
            assert!(self.model.n_vars() >= candidates.len());
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

        let br = FirstBranchingRule {
            model: model.clone_for_plugins(),
        };
        let solved = model
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();

        assert!(solved.n_nodes() > 1);
    }

    struct CustomBranchingRule {
        model: Model<Solving>,
    }

    impl BranchRule for CustomBranchingRule {
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
            self.model.create_child();
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

        let br = CustomBranchingRule {
            model: model.clone_for_plugins(),
        };
        let solved = model
            .include_branch_rule("", "", 100000, 1000, 1., Box::new(br))
            .solve();

        assert!(solved.n_nodes() > 1);
    }
}
