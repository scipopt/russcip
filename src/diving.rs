use crate::scip::ScipPtr;
use crate::{ffi, scip_call, scip_call_panic, Retcode, Row, Variable};
use std::rc::Rc;

/// Struct giving access to methods allowed in diving mode
pub struct Diver {
    pub(crate) scip: Rc<ScipPtr>,
}

impl Diver {
    /// Changes the lower bound of a variable in the current dive
    pub fn chg_var_lb(&mut self, var: &Variable, new_bound: f64) {
        scip_call_panic! { ffi::SCIPchgVarLbDive(self.scip.raw, var.inner(), new_bound) }
    }

    /// Changes the upper bound of a variable in the current dive
    pub fn chg_var_ub(&mut self, var: &Variable, new_bound: f64) {
        scip_call_panic! { ffi::SCIPchgVarUbDive(self.scip.raw, var.inner(), new_bound) }
    }

    /// Changes the objective value of a variable in the current dive
    pub fn chg_var_obj(&mut self, var: &Variable, new_obj: f64) {
        scip_call_panic! { ffi::SCIPchgVarObjDive(self.scip.raw, var.inner(), new_obj) }
    }

    /// Solves the diving LP
    ///
    /// # Arguments
    /// - `iteration_limit`: the maximum number of iterations to be performed, or `None` for no limit
    ///
    /// # Returns
    /// - `Ok(lp_solved)`: whether the LP was successfully solved to optimality
    /// - `Err(retcode)`: if an error occurred
    pub fn solve_lp(&mut self, iteration_limit: Option<usize>) -> Result<bool, Retcode> {
        let mut limit = -1;
        if let Some(iterations) = iteration_limit {
            limit = iterations.try_into().unwrap();
        }
        let mut lperror = 0;
        let mut lpsolved = 0;

        scip_call! { ffi::SCIPsolveDiveLP(self.scip.raw, limit, &mut lperror, &mut lpsolved) }

        if lperror != 0 {
            return Err(Retcode::LpError);
        }

        Ok(lpsolved != 0)
    }

    /// Adds a row to the diving LP
    pub fn add_row(&mut self, row: &Row) {
        scip_call_panic! { ffi::SCIPaddRowDive(self.scip.raw, row.inner()) }
    }

    /// Change a row's lhs in the diving LP
    pub fn chg_row_lhs(&mut self, row: &Row, new_lhs: f64) {
        scip_call_panic! { ffi::SCIPchgRowLhsDive(self.scip.raw, row.inner(), new_lhs) }
    }

    /// Change a row's rhs in the diving LP
    pub fn chg_row_rhs(&mut self, row: &Row, new_rhs: f64) {
        scip_call_panic! { ffi::SCIPchgRowRhsDive(self.scip.raw, row.inner(), new_rhs) }
    }

    /// Gets the variable objective value in the diving LP
    pub fn var_obj(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetVarObjDive(self.scip.raw, var.inner()) }
    }

    /// Gets the variable lower bound in the diving LP
    pub fn var_lb(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetVarLbDive(self.scip.raw, var.inner()) }
    }

    /// Gets the variable upper bound in the diving LP
    pub fn var_ub(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetVarUbDive(self.scip.raw, var.inner()) }
    }

    /// Gets the last branch-and-bound node (in the current run) number where diving was started
    pub fn last_dive_node(&self) -> usize {
        unsafe { ffi::SCIPgetLastDivenode(self.scip.raw) as usize }
    }

    /// Changes the cutoff bound in the diving LP
    pub fn chg_cutoff_bound(&mut self, cutoff: f64) {
        scip_call_panic! { ffi::SCIPchgCutoffboundDive(self.scip.raw, cutoff) }
    }
}

impl Drop for Diver {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { ffi::SCIPinDive(self.scip.raw) },
            1,
            "SCIP is expected to be in diving mode before Diver is dropped."
        );
        unsafe { ffi::SCIPendDive(self.scip.raw) };
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;
    use crate::prelude::{eventhdlr, row};
    use crate::{ffi, Eventhdlr, LPStatus, ModelWithProblem, ParamSetting};
    use crate::{Event, EventMask, SCIPEventhdlr, Solving};

    #[test]
    fn test_diver() {
        struct DivingTester;

        impl Eventhdlr for DivingTester {
            fn get_type(&self) -> EventMask {
                EventMask::NODE_SOLVED
            }

            fn execute(
                &mut self,
                mut model: Model<Solving>,
                _eventhdlr: SCIPEventhdlr,
                _event: Event,
            ) {
                let mut diver = model.start_diving();

                let vars = model.vars();
                for var in vars {
                    diver.chg_var_obj(&var, 0.0);
                    assert_eq!(diver.var_obj(&var), 0.0);
                }

                let result = diver.solve_lp(None).unwrap();
                assert!(result);

                assert_eq!(model.lp_status(), LPStatus::Optimal);
                assert!(model.lp_obj_val().abs() < 1e-6);

                let current_node = model.focus_node().number();
                assert_eq!(diver.last_dive_node(), current_node);

                diver.add_row(&model.add(row().eq(-1.0))); // unsatisfiable row
                diver.solve_lp(None).unwrap();
                assert_eq!(model.lp_status(), LPStatus::Infeasible);

                // Drop diver to force ending diving mode
                drop(diver);

                // Check that diving mode is ended
                assert_eq!(unsafe { ffi::SCIPinDive(model.scip.raw) }, 0);
            }
        }

        let mut model = Model::new()
            .include_default_plugins()
            .read_prob("data/test/simple.mps")
            .unwrap()
            .hide_output()
            .set_presolving(ParamSetting::Off)
            .set_separating(ParamSetting::Off)
            .set_heuristics(ParamSetting::Off);

        model.add(eventhdlr(DivingTester));
        model.solve();
    }
}
