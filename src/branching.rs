use crate::ffi;
use crate::retcode::Retcode;

pub struct BranchRule {}

impl BranchRule {
    pub fn execute(&mut self, candidates: Vec<BranchingCandidate>) -> Retcode {
        Retcode::Okay
    }
}

pub struct BranchingCandidate {
    pub var_id: usize,
    pub lp_sol_val: f64,
}

#[cfg(test)]
mod tests {
    use crate::branching::{BranchRule, BranchingCandidate};
    use crate::model::Model;
    use crate::retcode::Retcode;

    #[test]
    fn test_branching() {
        // build default model
        let br = BranchRule{};
        let model = Model::new()
            .include_branch_rule("", "", 100000, 1000, 1., br)
            .include_default_plugins()
            .create_prob("");

        // solve model
        model.solve();
    }
}
