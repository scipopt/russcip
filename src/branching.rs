use crate::ffi;
use crate::retcode::Retcode;

pub trait BranchRule {
    fn execute(&self, candidates: Vec<BranchingCandidate>) -> BranchingResult {
        BranchingResult::DidNotRun
    }
}

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
}

#[cfg(test)]
mod tests {
    use crate::branching::BranchRule;
    use crate::model::{Model, ParamSetting};

    struct SimpleBranchingRule;
    impl BranchRule for SimpleBranchingRule {}

    #[test]
    fn test_branching() {
        let mut br = SimpleBranchingRule {};

        // create model from miplib instance gen-ip054
        let model = Model::new()
            // .hide_output()
            .set_heuristics(ParamSetting::Off)
            .include_branch_rule("", "", 100000, 1000, 1., &mut br)
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        // solve model
        model.solve();
    }
}
