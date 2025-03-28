use crate::{ffi, Model, Solving};
use scip_sys::SCIP_Result;

/// A trait for defining custom branching rules.
pub trait BranchRule {
    /// Executes the branching rule on the given candidates and returns the result.
    ///
    /// # Arguments
    /// * `model` - the current model of the SCIP instance in `Solving` stage.
    /// * `branchrule` - the internal SCIP branch rule.
    /// * `candidates` - the branching candidates.
    ///
    /// # Returns
    ///
    /// * `BranchingResult` indicating the result of the branching rule.
    fn execute(
        &mut self,
        model: Model<Solving>,
        branchrule: SCIPBranchRule,
        candidates: Vec<BranchingCandidate>,
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

/// A wrapper struct for the internal ffi::SCIP_BRANCHRULE
pub struct SCIPBranchRule {
    pub(crate) raw: *mut ffi::SCIP_BRANCHRULE,
}

impl SCIPBranchRule {
    /// Returns the internal raw pointer of the branch rule.
    pub fn inner(&self) -> *mut ffi::SCIP_BRANCHRULE {
        self.raw
    }

    /// Returns the name of the branch rule.
    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = ffi::SCIPbranchruleGetName(self.raw);
            let name = std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap();
            name.to_string()
        }
    }

    /// Returns the description of the branch rule.
    pub fn desc(&self) -> String {
        unsafe {
            let desc_ptr = ffi::SCIPbranchruleGetDesc(self.raw);
            let desc = std::ffi::CStr::from_ptr(desc_ptr).to_str().unwrap();
            desc.to_string()
        }
    }

    /// Returns the priority of the branch rule.
    pub fn priority(&self) -> i32 {
        unsafe { ffi::SCIPbranchruleGetPriority(self.raw) }
    }

    /// Returns the maxdepth of the branch rule.
    pub fn maxdepth(&self) -> i32 {
        unsafe { ffi::SCIPbranchruleGetMaxdepth(self.raw) }
    }

    /// Returns the maxbounddist of the branch rule.
    pub fn maxbounddist(&self) -> f64 {
        unsafe { ffi::SCIPbranchruleGetMaxbounddist(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelWithProblem;
    use crate::prelude::{branchrule, cons};
    use crate::{model::Model, status::Status, Solving};

    struct FirstChoosingBranchingRule {
        pub chosen: Option<BranchingCandidate>,
    }

    impl BranchRule for FirstChoosingBranchingRule {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            branchrule: SCIPBranchRule,
            candidates: Vec<BranchingCandidate>,
        ) -> BranchingResult {
            self.chosen = Some(candidates[0].clone());
            assert_eq!(branchrule.name(), "FirstChoosingBranchingRule");
            BranchingResult::DidNotRun
        }
    }

    #[test]
    fn choosing_first_branching_rule() {
        let br = FirstChoosingBranchingRule { chosen: None };

        let mut model = Model::new()
            .set_longint_param("limits/nodes", 2) // only call brancher once
            .unwrap()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        model.add(branchrule(br).name("FirstChoosingBranchingRule"));

        let solved = model.solve();
        assert_eq!(solved.status(), Status::NodeLimit);
    }

    struct CuttingOffBranchingRule;

    impl BranchRule for CuttingOffBranchingRule {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            _candidates: Vec<BranchingCandidate>,
        ) -> BranchingResult {
            BranchingResult::CutOff
        }
    }

    #[test]
    fn cutting_off_branching_rule() {
        let br = CuttingOffBranchingRule {};

        // create model from miplib instance gen-ip054
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();
        model.add(branchrule(br).maxdepth(10));
        let model = model.solve();
        assert_eq!(model.n_nodes(), 1);
    }

    struct FirstBranchingRule;

    impl BranchRule for FirstBranchingRule {
        fn execute(
            &mut self,
            model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            candidates: Vec<BranchingCandidate>,
        ) -> BranchingResult {
            assert!(model.n_vars() >= candidates.len());
            BranchingResult::BranchOn(candidates[0].clone())
        }
    }

    #[test]
    fn first_branching_rule() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = FirstBranchingRule;
        model.add(branchrule(br).name("FirstBranchingRule").maxdepth(1000));
        let solved = model.solve();

        assert!(solved.n_nodes() > 1);
    }

    struct CustomBranchingRule;

    impl BranchRule for CustomBranchingRule {
        fn execute(
            &mut self,
            mut model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            _candidates: Vec<BranchingCandidate>,
        ) -> BranchingResult {
            let child1 = model.create_child();
            let child2 = model.create_child();

            let vars = model.vars();
            model.add_cons_node(
                child1,
                cons().
                eq(0.0).
                coef(&vars[0], 1.).
                coef(&vars[1], -1.)
            );

            model.add_cons_node(
                child2,
                cons().
                eq(1.0).
                coef(&vars[0], 1.).
                coef(&vars[1], 1.)
            );

            // model.add_cons_node(child1, vec![&vars[0], &vars[1]], &[1., -1.], 0., 0., "eq");
            // model.add_cons_node(child2, vec![&vars[0], &vars[1]], &[1., 1.], 0., 1., "diff");
            BranchingResult::CustomBranching
        }
    }

    #[test]
    fn custom_branching_rule() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = CustomBranchingRule;
        model.add(branchrule(br));
        model.add_var(0., 1., 1., "x", crate::variable::VarType::Binary);
        model.add_var(0., 1., 1., "y", crate::variable::VarType::Binary);
        let solved = model.solve();

        assert!(solved.n_nodes() > 1);
    }

    struct HighestBoundBranchRule;

    impl BranchRule for HighestBoundBranchRule {
        fn execute(
            &mut self,
            model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            candidates: Vec<BranchingCandidate>,
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
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = HighestBoundBranchRule;
        model.add(branchrule(br));
        let solved = model.solve();

        assert!(solved.n_nodes() > 1);
    }

    struct InternalBranchRuleDataTester;

    impl BranchRule for InternalBranchRuleDataTester {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _branchrule: SCIPBranchRule,
            _candidates: Vec<BranchingCandidate>,
        ) -> BranchingResult {
            BranchingResult::DidNotRun
        }
    }

    #[test]
    fn test_internal_scip_branch_rule() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let br = InternalBranchRuleDataTester;
        model.add(branchrule(br).maxdepth(1));
        model.solve();
    }
}
