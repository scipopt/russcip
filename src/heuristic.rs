use scip_sys::SCIP_Result;
use std::ops::{BitOr, BitOrAssign};

use crate::ffi;

/// A trait for defining custom primal heuristics.
pub trait Heuristic {
    /// Executes the heuristic.
    ///
    /// # Arguments
    /// * `timing` - the timing mask of the heuristic's execution
    /// * `node_inf` - whether the current node is infeasible
    ///
    /// # Returns
    ///
    /// * `HeurResult::FoundSol` if a new incumbent solution was found
    /// * `HeurResult::NoSolFound` if no new incumbent solution was found
    /// * `HeurResult::DidNotRun` if the heuristic was not executed
    /// * `HeurResult::Delayed` if the heuristic is delayed (skipped but should be called again)
    fn execute(&mut self, timing: HeurTiming, node_inf: bool) -> HeurResult;
}

/// The result of a primal heuristic execution.
#[derive(Debug, PartialEq, Eq)]
pub enum HeurResult {
    /// The heuristic found a new incumbent solution.
    FoundSol,
    /// The heuristic did not find a new solution.
    NoSolFound,
    /// The heuristic was not executed.
    DidNotRun,
    /// The heuristic is delayed (skipped but should be called again).
    Delayed,
}

/// The Heur represents different timing masks for the execution of a heuristic.
#[derive(Debug, Copy, Clone)]
pub struct HeurTiming(u64);

impl HeurTiming {
    /// call heuristic before the processing of the node starts
    pub const BEFORE_NODE: Self = HeurTiming(0x001);
    /// call heuristic after each LP solving during cut-and-price loop
    pub const DURING_LP_LOOP: Self = HeurTiming(0x002);
    /// call heuristic after the cut-and-price loop was finished
    pub const AFTER_LP_LOOP: Self = HeurTiming(0x004);
    /// call heuristic after the processing of a node with solved LP was finished
    pub const AFTER_LP_NODE: Self = HeurTiming(0x008);
    /// call heuristic after the processing of a node without solved LP was finished
    pub const AFTER_PSEUDO_NODE: Self = HeurTiming(0x010);
    /// call heuristic after the processing of the last node in the current plunge was finished, and only if the LP was solved for this node
    pub const AFTER_LP_PLUNGE: Self = HeurTiming(0x020);
    /// call heuristic after the processing of the last node in the current plunge was finished, and only if the LP was not solved for this node
    pub const AFTER_PSEUDO_PLUNGE: Self = HeurTiming(0x040);
    /// call heuristic during pricing loop
    pub const DURING_PRICING_LOOP: Self = HeurTiming(0x080);
    /// call heuristic before presolving
    pub const BEFORE_PRESOL: Self = HeurTiming(0x100);
    /// call heuristic during presolving loop
    pub const DURING_PRESOL_LOOP: Self = HeurTiming(0x200);
    /// call heuristic after propagation which is performed before solving the LP
    pub const AFTER_PROP_LOOP: Self = HeurTiming(0x400);
}

impl BitOr for HeurTiming {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        HeurTiming(self.0 | rhs.0)
    }
}

impl BitOrAssign for HeurTiming {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<HeurTiming> for u32 {
    fn from(mask: HeurTiming) -> Self {
        mask.0 as u32
    }
}

impl From<u32> for HeurTiming {
    fn from(mask: u32) -> Self {
        HeurTiming(mask as u64)
    }
}

impl From<HeurResult> for SCIP_Result {
    fn from(val: HeurResult) -> Self {
        match val {
            HeurResult::FoundSol => ffi::SCIP_Result_SCIP_FOUNDSOL,
            HeurResult::NoSolFound => ffi::SCIP_Result_SCIP_DIDNOTFIND,
            HeurResult::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            HeurResult::Delayed => ffi::SCIP_Result_SCIP_DELAYED,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Model, ModelWithProblem, ProblemOrSolving, Solving};

    use super::*;

    struct NoSolutionFoundHeur;

    impl Heuristic for NoSolutionFoundHeur {
        fn execute(&mut self, _timing: HeurTiming, _node_inf: bool) -> HeurResult {
            HeurResult::NoSolFound
        }
    }

    #[test]
    fn test_heur() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let heur = NoSolutionFoundHeur;
        let mut timing = HeurTiming::BEFORE_PRESOL;
        timing |= HeurTiming::AFTER_PROP_LOOP;
        model
            .include_heur(
                "no_sol_found_heur",
                "",
                9999999,
                'n',
                1,
                0,
                -1,
                timing,
                false,
                Box::new(heur),
            )
            .solve();
    }

    struct ImpostorHeur;

    impl Heuristic for ImpostorHeur {
        fn execute(&mut self, _timing: HeurTiming, _node_inf: bool) -> HeurResult {
            HeurResult::FoundSol
        }
    }

    #[test]
    #[should_panic]
    fn impostor_heur() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let heur = ImpostorHeur;
        model
            .include_heur(
                "impostor_heur",
                "",
                9999999,
                'n',
                1,
                0,
                -1,
                HeurTiming::BEFORE_NODE | HeurTiming::AFTER_LP_NODE,
                false,
                Box::new(heur),
            )
            .solve();
    }

    struct DelayedHeur;

    impl Heuristic for DelayedHeur {
        fn execute(&mut self, _timing: HeurTiming, _node_inf: bool) -> HeurResult {
            HeurResult::Delayed
        }
    }

    #[test]
    fn delayed_heur() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let heur = DelayedHeur;
        model
            .include_heur(
                "delayed_heur",
                "",
                9999999,
                'n',
                1,
                0,
                -1,
                HeurTiming::BEFORE_NODE,
                false,
                Box::new(heur),
            )
            .solve();
    }

    struct DidNotRunHeur;

    impl Heuristic for DidNotRunHeur {
        fn execute(&mut self, _timing: HeurTiming, _node_inf: bool) -> HeurResult {
            HeurResult::DidNotRun
        }
    }

    #[test]
    fn did_not_run_heur() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let heur = DidNotRunHeur;
        model
            .include_heur(
                "did_not_run_heur",
                "",
                9999999,
                'n',
                1,
                0,
                -1,
                HeurTiming::BEFORE_NODE,
                false,
                Box::new(heur),
            )
            .solve();
    }

    struct FoundSolHeur {
        model: Model<Solving>,
    }

    impl Heuristic for FoundSolHeur {
        fn execute(&mut self, _timing: HeurTiming, _node_inf: bool) -> HeurResult {
            let sol = self.model.create_sol();
            for var in self.model.vars() {
                sol.set_val(var, 1.0);
            }
            assert_eq!(sol.obj_val(), 7.0);
            assert_eq!(self.model.add_sol(sol), Ok(()));
            HeurResult::FoundSol
        }
    }

    #[test]
    fn found_sol_heur() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let heur = FoundSolHeur {
            model: model.clone_for_plugins(),
        };
        model
            .include_heur(
                "found_sol_heur",
                "",
                9999999,
                'n',
                1,
                0,
                -1,
                HeurTiming::BEFORE_NODE,
                false,
                Box::new(heur),
            )
            .solve();
    }
}
