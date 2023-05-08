use crate::ffi;

pub trait BranchRule {
    fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> BranchingResult;
}

#[derive(Debug, Clone)]
pub enum BranchingResult {
    DidNotRun,
    BranchOn(BranchingCandidate),
    CutOff,
    CustomBranching,
}

impl From<BranchingResult> for u32 {
    fn from(val: BranchingResult) -> Self {
        match val {
            BranchingResult::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            BranchingResult::BranchOn(_) => ffi::SCIP_Result_SCIP_BRANCHED,
            BranchingResult::CutOff => ffi::SCIP_Result_SCIP_CUTOFF,
            BranchingResult::CustomBranching => ffi::SCIP_Result_SCIP_BRANCHED,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchingCandidate {
    pub(crate) var_ptr: *mut ffi::SCIP_VAR,
    pub lp_sol_val: f64,
    pub frac: f64,
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
        let mut br = FirstChoosingBranchingRule { chosen: None };

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
        assert!(br.chosen.is_some());
        let candidate = br.chosen.unwrap();
        assert!(candidate.lp_sol_val.fract() > 0.);
        assert!(candidate.frac > 0. && candidate.frac < 1.);
    }



    struct CuttingOffBranchingRule;

    impl BranchRule for CuttingOffBranchingRule {
        fn execute(&mut self, _candidates: Vec<BranchingCandidate>) -> BranchingResult {
            BranchingResult::CutOff
        }
    }

    #[test]
    fn cutting_off_branching_rule() {
        let mut br = CuttingOffBranchingRule {};

        // create model from miplib instance gen-ip054
        let model = Model::new()
            .hide_output()
            .include_branch_rule("", "", 100000, 1000, 1., &mut br)
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        // solve model
        let solved = model.solve();

        //TODO test number of nodes == 1
    }

}
