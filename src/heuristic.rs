use scip_sys::SCIP_Result;
use std::ops::{BitOr, BitOrAssign};
use std::rc::Rc;

use crate::scip::ScipPtr;
use crate::{Model, Solving, ffi};

/// A primal heuristic that is part of the model, providing access to its
/// runtime statistics (e.g. how often it ran and how many solutions it found).
///
/// Obtain one via [`ModelWithProblem::find_heur`](crate::ModelWithProblem::find_heur).
#[derive(Clone)]
pub struct Heur {
    /// A pointer to the underlying `SCIP_HEUR` C struct.
    pub(crate) raw: *mut ffi::SCIP_HEUR,
    /// A reference to the SCIP instance that owns this heuristic (to prevent
    /// freeing the model while the heuristic is live).
    #[allow(dead_code)]
    pub(crate) scip: Rc<ScipPtr>,
}

impl Heur {
    /// Returns a pointer to the underlying `SCIP_HEUR` C struct.
    pub fn inner(&self) -> *mut ffi::SCIP_HEUR {
        self.raw
    }

    /// Returns the name of the heuristic.
    pub fn name(&self) -> String {
        unsafe {
            let name = ffi::SCIPheurGetName(self.raw);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }

    /// Returns the number of times the heuristic was called during the solving process.
    pub fn n_calls(&self) -> usize {
        (unsafe { ffi::SCIPheurGetNCalls(self.raw) }) as usize
    }

    /// Returns the number of solutions the heuristic found during the solving process.
    pub fn n_sols_found(&self) -> usize {
        (unsafe { ffi::SCIPheurGetNSolsFound(self.raw) }) as usize
    }

    /// Returns the number of new best (incumbent) solutions the heuristic found
    /// during the solving process.
    pub fn n_best_sols_found(&self) -> usize {
        (unsafe { ffi::SCIPheurGetNBestSolsFound(self.raw) }) as usize
    }
}

/// A trait for defining custom primal heuristics.
pub trait Heuristic {
    /// Executes the heuristic.
    ///
    /// # Arguments
    /// * `model` - the current model of the SCIP instance in `Solving` stage
    /// * `timing` - the timing mask of the heuristic's execution
    /// * `node_inf` - whether the current node is infeasible
    ///
    /// # Returns
    ///
    /// * `HeurResult::FoundSol` if a new incumbent solution was found
    /// * `HeurResult::NoSolFound` if no new incumbent solution was found
    /// * `HeurResult::DidNotRun` if the heuristic was not executed
    /// * `HeurResult::Delayed` if the heuristic is delayed (skipped but should be called again)
    fn execute(&mut self, model: Model<Solving>, timing: HeurTiming, node_inf: bool) -> HeurResult;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    use super::*;
    use crate::prelude::heur;
    use crate::{Model, ModelWithProblem, ProblemOrSolving};

    #[test]
    fn find_heur_by_name() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        let heur = model
            .find_heur("completesol")
            .expect("completesol is a default heuristic");
        assert_eq!(heur.name(), "completesol");
        // completesol only runs given a partial solution, so on this instance
        // all of its counters stay at zero.
        assert_eq!(heur.n_calls(), 0);
        assert_eq!(heur.n_sols_found(), 0);
        assert_eq!(heur.n_best_sols_found(), 0);

        assert!(model.find_heur("definitely_not_a_heuristic").is_none());
    }

    struct NoSolutionFoundHeur;

    impl Heuristic for NoSolutionFoundHeur {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _timing: HeurTiming,
            _node_inf: bool,
        ) -> HeurResult {
            HeurResult::NoSolFound
        }
    }

    #[test]
    fn test_heur() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let hr = NoSolutionFoundHeur;
        let mut timing = HeurTiming::BEFORE_PRESOL;
        timing |= HeurTiming::AFTER_PROP_LOOP;
        model.add(
            heur(hr)
                .name("no_sol_found_heur")
                .timing(timing)
                .dispchar('n'),
        );
        model.solve();
    }

    struct ImpostorHeur;

    impl Heuristic for ImpostorHeur {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _timing: HeurTiming,
            _node_inf: bool,
        ) -> HeurResult {
            HeurResult::FoundSol
        }
    }

    #[test]
    #[should_panic]
    fn impostor_heur() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let h = ImpostorHeur;

        model.add(
            heur(h)
                .name("impostor_heur")
                .timing(HeurTiming::BEFORE_NODE | HeurTiming::AFTER_LP_NODE),
        );
        model.solve();
    }

    struct DelayedHeur;

    impl Heuristic for DelayedHeur {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _timing: HeurTiming,
            _node_inf: bool,
        ) -> HeurResult {
            HeurResult::Delayed
        }
    }

    #[test]
    fn delayed_heur() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let h = DelayedHeur;
        model.add(heur(h).name("delayed_heur").timing(HeurTiming::BEFORE_NODE));
        model.solve();
    }

    struct DidNotRunHeur;

    impl Heuristic for DidNotRunHeur {
        fn execute(
            &mut self,
            _model: Model<Solving>,
            _timing: HeurTiming,
            _node_inf: bool,
        ) -> HeurResult {
            HeurResult::DidNotRun
        }
    }

    #[test]
    fn did_not_run_heur() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let h = DidNotRunHeur;
        model.add(heur(h).name("did_not_run_heur"));
        model.solve();
    }

    struct FoundSolHeur;

    impl Heuristic for FoundSolHeur {
        fn execute(
            &mut self,
            model: Model<Solving>,
            _timing: HeurTiming,
            _node_inf: bool,
        ) -> HeurResult {
            let sol = model.create_sol();
            for var in model.vars() {
                sol.set_val(&var, 1.0);
            }
            assert_eq!(sol.obj_val(), 7.0);
            assert_eq!(model.add_sol(sol), Ok(()));
            HeurResult::FoundSol
        }
    }

    #[test]
    fn found_sol_heur() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let h = FoundSolHeur;
        model.add(heur(h).name("found_sol_heur"));
        model.solve();
    }
}
