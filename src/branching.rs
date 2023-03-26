use crate::ffi;

pub trait BranchRule {
    fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> BranchingResult;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BranchingResult {
    DidNotRun,
    Branched,
}

impl From<BranchingResult> for u32 {
    fn from(val: BranchingResult) -> Self {
        match val {
            BranchingResult::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            BranchingResult::Branched => ffi::SCIP_Result_SCIP_BRANCHED,
        }
    }
}

pub struct BranchingCandidate {
    pub var_id: usize,
    pub lp_sol_val: f64,
    pub frac: f64,
}

impl BranchingCandidate {
    pub fn new(var_id: usize, lp_sol_val: f64) -> Self {
        let frac = lp_sol_val.fract();
        Self {
            var_id,
            lp_sol_val,
            frac,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::Model, status::Status};

    struct PanickingBranchingRule;
    impl BranchRule for PanickingBranchingRule {
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
            panic!("Not implemented")
        }
    }

    #[test]
    #[should_panic]
    fn test_branching() {
        let mut br = PanickingBranchingRule {};

        // create model from miplib instance gen-ip054
        let model = Model::new()
            .hide_output()
            .include_branch_rule("", "", 100000, 1000, 1., &mut br)
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        // solve model
        model.solve();
    }

    struct TestBranchingRule {
        pub called: bool,
    }

    impl BranchRule for TestBranchingRule {
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
            self.called = true;
            BranchingResult::DidNotRun
        }
    }

    #[test]
    fn test_branching_rule() {
        let mut br = TestBranchingRule { called: false };

        // create model from miplib instance gen-ip054
        let model = Model::new()
            .set_longint_param("limits/nodes", 2) // only call brancher once
            .unwrap()
            .hide_output()
            .include_branch_rule("", "", 100000, 1000, 1., &mut br)
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        // solve model
        let solved = model.solve();
        assert_eq!(solved.get_status(), Status::NodeLimit);
        assert!(br.called);
    }
}
