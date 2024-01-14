use std::fmt;
use std::rc::Rc;

use crate::variable::Variable;
use crate::{ffi, scip_call_panic};

/// A wrapper for a SCIP solution.
#[derive(PartialEq, Eq)]
pub struct Solution {
    pub(crate) scip_ptr: *mut ffi::SCIP,
    pub(crate) raw: *mut ffi::SCIP_SOL,
}

impl Solution {
    /// Returns the objective value of the solution.
    pub fn obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.scip_ptr, self.raw) }
    }

    /// Returns the value of a variable in the solution.
    pub fn val(&self, var: Rc<Variable>) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var.raw) }
    }

    /// Sets the value of a variable in the solution.
    pub fn set_val(&self, var: Rc<Variable>, val: f64) {
        scip_call_panic!(ffi::SCIPsetSolVal(self.scip_ptr, self.raw, var.raw, val));
    }
}

impl fmt::Debug for Solution {
    /// Formats the solution for debugging purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj_val = self.obj_val();
        writeln!(f, "Solution with obj val: {obj_val}")?;
        let vars = unsafe { ffi::SCIPgetVars(self.scip_ptr) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.scip_ptr) };
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var) };
            let eps = unsafe { ffi::SCIPepsilon(self.scip_ptr) };
            if val > eps || val < -eps {
                let name_ptr = unsafe { ffi::SCIPvarGetName(var) };
                // from CString
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap() };
                writeln!(f, "Var {name}={val}")?;
            }
        }
        Ok(())
    }
}

/// Represents and error that can occur when adding a solution.
#[derive(Debug, PartialEq, Eq)]
pub enum SolError {
    /// The solution is infeasible.
    Infeasible,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn sol_methods() {
        let model = Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        let status = model.status();
        assert_eq!(status, Status::Optimal);

        //test solution values
        let sol = model.best_sol().unwrap();

        let debug_str = format!("{:?}", sol);
        assert!(debug_str.contains("Solution with obj val: 200"));
        assert!(debug_str.contains("Var t_x1=40"));
        assert!(debug_str.contains("Var t_x2=20"));

        let vars = model.vars();
        assert_eq!(sol.val(vars[0].clone()), 40.);
        assert_eq!(sol.val(vars[1].clone()), 20.);

        assert_eq!(sol.obj_val(), model.obj_val());
    }
}
