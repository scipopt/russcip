use crate::scip::ScipPtr;
use crate::{Retcode, Row, Variable, ffi, scip_call, scip_call_panic};
use std::rc::Rc;

/// Struct giving access to methods allowed in probing mode
pub struct Prober {
    pub(crate) scip: Rc<ScipPtr>,
}

impl Prober {
    /// Creates a new probing (sub-)node, whose changes can be undone by backtracking to a higher node
    /// in the probing path with a call to the `backtrack()` method.
    pub fn new_node(&mut self) {
        scip_call_panic! { ffi::SCIPnewProbingNode(self.scip.raw) }
    }

    /// Returns the current probing depth
    pub fn depth(&self) -> usize {
        unsafe { ffi::SCIPgetProbingDepth(self.scip.raw) }
            .try_into()
            .expect("Invalid depth value")
    }

    /// Undoes all changes to the problem applied in probing up to the given probing depth;
    /// the changes of the probing node of the given probing depth are the last ones that remain active;
    /// changes that were applied before calling `new_node()` cannot be undone
    pub fn backtrack(&mut self, depth: usize) {
        assert!(
            depth < self.depth(),
            "Probing depth must be less than the current probing depth."
        );
        scip_call_panic! { ffi::SCIPbacktrackProbing(self.scip.raw, depth.try_into().unwrap()) }
    }

    /// Changes the lower bound of a variable in the current probing node
    pub fn chg_var_lb(&mut self, var: &Variable, new_bound: f64) {
        scip_call_panic! { ffi::SCIPchgVarLbProbing(self.scip.raw, var.inner(), new_bound) }
    }

    /// Changes the upper bound of a variable in the current probing node
    pub fn chg_var_ub(&mut self, var: &Variable, new_bound: f64) {
        scip_call_panic! { ffi::SCIPchgVarUbProbing(self.scip.raw, var.inner(), new_bound) }
    }

    /// Retrieves the objective value of a variable in the current probing node
    pub fn var_obj(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetVarObjProbing(self.scip.raw, var.inner()) }
    }

    /// Fixes a variable to a value in the current probing node
    pub fn fix_var(&mut self, var: &Variable, value: f64) {
        scip_call_panic! { ffi::SCIPfixVarProbing(self.scip.raw, var.inner(), value) }
    }

    /// Changes the objective value of a variable in the current probing node
    pub fn chg_var_obj(&mut self, var: &Variable, new_obj: f64) {
        scip_call_panic! { ffi::SCIPchgVarObjProbing(self.scip.raw, var.inner(), new_obj) }
    }

    /// Returns whether the probing subproblem objective function has been changed
    pub fn is_obj_changed(&self) -> bool {
        unsafe { ffi::SCIPisObjChangedProbing(self.scip.raw) != 0 }
    }

    /// Applies domain propagation on the probing subproblem; the propagated domains of the variables
    /// can be accessed with the usual bound accessing calls to `var.lb_local()` and `var.ub_local()`
    ///
    /// # Arguments
    /// - `max_rounds`: the maximum number of rounds to be performed, or `None` for no limit
    ///
    /// # Returns
    /// A tuple (`cutoff`, `nreductions_found`)
    /// - `cutoff`: whether a cutoff was detected
    /// - `nreductions_found`: the number of reductions found
    pub fn propagate(&mut self, max_rounds: Option<usize>) -> (bool, usize) {
        let mut cutoff = 0;
        let mut nreductions_found = 0;
        let mut r = -1;
        if let Some(rounds) = max_rounds {
            r = rounds.try_into().unwrap();
        }

        scip_call_panic! {
            ffi::SCIPpropagateProbing(self.scip.raw, r, &mut cutoff, &mut nreductions_found)
        }

        (cutoff != 0, nreductions_found.try_into().unwrap())
    }

    /// Applies domain propagation on the probing subproblem; only propagations of the binary variables
    /// fixed at the current probing node that are triggered by the implication graph and the clique
    /// table are applied; the propagated domains of the variables can be accessed with the usual
    /// bound accessing calls to `var.lb_local()` and `var.ub_local()`
    ///
    /// # Returns
    /// - `cutoff`: whether a cutoff was detected
    pub fn propagate_implications(&mut self) -> bool {
        let mut cutoff = 0;
        scip_call_panic! {
            ffi::SCIPpropagateProbingImplications(self.scip.raw, &mut cutoff)
        }

        cutoff != 0
    }

    /// Solves the probing subproblem; the solution can be accessed with the `model.current_val()` method
    ///
    /// # Arguments
    /// - `iteration_limit`: the maximum number of iterations to be performed, or `None` for no limit
    ///
    /// # Returns
    /// - `cutoff`: whether a cutoff was detected
    pub fn solve_lp(&mut self, iteration_limit: Option<usize>) -> Result<bool, Retcode> {
        if !self.scip.is_lp_constructed() {
            self.scip.construct_lp()?;
        }

        let mut limit = -1;
        if let Some(iterations) = iteration_limit {
            limit = iterations.try_into().unwrap();
        }
        let mut cutoff = 0;
        let mut lperror = 0;
        scip_call! { ffi::SCIPsolveProbingLP(self.scip.raw, limit, &mut cutoff, &mut lperror) }

        if lperror != 0 {
            return Err(Retcode::LpError);
        }

        Ok(cutoff != 0)
    }

    /// Solves the probing subproblem with pricing; the solution can be accessed
    /// with the `model.current_val()` method.
    ///
    /// # Arguments
    /// - `max_pricing_rounds`: the maximum number of pricing rounds to be performed, or `None` for no limit
    ///
    /// # Returns
    /// - `cutoff`: whether a cutoff was detected
    pub fn solve_lp_with_pricing(
        &mut self,
        max_pricing_rounds: Option<usize>,
    ) -> Result<bool, Retcode> {
        if !self.scip.is_lp_constructed() {
            self.scip.construct_lp()?;
        }

        let mut rounds = -1;
        if let Some(r) = max_pricing_rounds {
            rounds = r.try_into().unwrap();
        }
        let mut cutoff = 0;
        let mut lperror = 0;

        // set a default for now to communicate the current state, any further needed communication
        // can be done by sharing data between plugins
        const PRETENDATROOT: u32 = 0;

        // enable always for now, to avoid unnecessary complexity
        const DISPLAYINFO: u32 = 1;

        scip_call! {
            ffi::SCIPsolveProbingLPWithPricing(
                self.scip.raw,
                PRETENDATROOT,
                DISPLAYINFO,
                rounds,
                &mut cutoff,
                &mut lperror,
            )
        }

        if lperror != 0 {
            return Err(Retcode::LpError);
        }

        Ok(cutoff != 0)
    }

    /// Adds a row to the probing subproblem
    pub fn add_row(&mut self, row: &Row) {
        scip_call_panic! { ffi::SCIPaddRowProbing(self.scip.raw, row.inner(),) }
    }
}

impl Drop for Prober {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { ffi::SCIPinProbing(self.scip.raw) },
            1,
            "SCIP is expected to be in probing mode before Prober is dropped."
        );
        unsafe { ffi::SCIPendProbing(self.scip.raw) };
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;
    use crate::prelude::{eventhdlr, row};
    use crate::{Event, EventMask, SCIPEventhdlr, Solving};
    use crate::{Eventhdlr, ModelWithProblem, ParamSetting, ffi};

    #[test]
    fn test_prober() {
        struct ProbingTester;

        impl Eventhdlr for ProbingTester {
            fn get_type(&self) -> EventMask {
                EventMask::NODE_SOLVED
            }

            fn execute(
                &mut self,
                mut model: Model<Solving>,
                _eventhdlr: SCIPEventhdlr,
                _event: Event,
            ) {
                let mut prober = model.start_probing();
                assert!(!prober.is_obj_changed());

                let vars = model.vars();
                for var in vars {
                    prober.chg_var_obj(&var, 0.0);
                }
                assert!(prober.is_obj_changed());

                prober.solve_lp(None).unwrap();

                assert!(model.lp_obj_val().abs() < 1e-6);

                drop(prober);

                // have to use unsafe here as the method is not available in the public API
                assert_eq!(unsafe { ffi::SCIPinProbing(model.scip_ptr()) }, 0);
            }
        }

        let mut model = Model::new()
            .include_default_plugins()
            .read_prob("data/test/simple.mps")
            .unwrap()
            .hide_output()
            .set_presolving(ParamSetting::Off)
            .set_separating(ParamSetting::Off)
            .set_heuristics(ParamSetting::Off)
            .set_param("branching/pscost/priority", 100000);

        model.add(eventhdlr(ProbingTester));
        model.solve();
    }

    #[test]
    fn test_probing_add_row() {
        struct ProbingAddRowTester;

        impl Eventhdlr for ProbingAddRowTester {
            fn get_type(&self) -> EventMask {
                EventMask::NODE_SOLVED
            }

            fn execute(
                &mut self,
                mut model: Model<Solving>,
                _eventhdlr: SCIPEventhdlr,
                _event: Event,
            ) {
                let mut prober = model.start_probing();
                let obj = model.lp_obj_val();
                assert!(obj < -25.0);
                let row = model.add(row().eq(-1.0)); // unsatisfiable row
                prober.add_row(&row);
                let _ = prober.solve_lp(None);
                let obj = model.lp_obj_val();
                assert!(obj.abs() > 1e15); // infeasible
            }
        }

        let mut model = Model::new()
            .include_default_plugins()
            .read_prob("data/test/simple.mps")
            .unwrap()
            .hide_output()
            .set_presolving(ParamSetting::Off)
            .set_separating(ParamSetting::Off)
            .set_heuristics(ParamSetting::Off)
            .set_param("branching/pscost/priority", 100000);

        model.add(eventhdlr(ProbingAddRowTester));

        model.solve();
    }
}
