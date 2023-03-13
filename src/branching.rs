use crate::ffi;
use crate::retcode::Retcode;

pub trait BranchRule {
    fn execute(&self, candidates: Vec<BranchingCandidate>) -> Retcode {
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

    struct SimpleBranchingRule;
    impl BranchRule for SimpleBranchingRule {}

    #[test]
    fn test_branching() {
        // build default model
        let mut  br = SimpleBranchingRule {};
        let model = Model::new()
            .include_branch_rule("", "", 100000, 1000, 1., &mut br)
            .include_default_plugins()
            .create_prob("");

        // solve model
        model.solve();
    }
}
