use std::fmt;
use std::ptr::NonNull;
use std::rc::Rc;

use crate::scip::ScipPtr;
use crate::variable::Variable;
use crate::{ffi, scip_call_panic};

/// A wrapper for a SCIP solution.
#[derive(Clone)]
pub struct Solution {
    pub(crate) scip_ptr: Rc<ScipPtr>,
    pub(crate) raw: NonNull<ffi::SCIP_SOL>,
}

impl Solution {
    /// Returns a raw pointer to the underlying `ffi::SCIP_SOL` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_SOL {
        self.raw.as_ptr()
    }

    /// Returns the objective value of the solution.
    pub fn obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.scip_ptr.raw, self.raw.as_ptr()) }
    }

    /// Returns the value of a variable in the solution.
    pub fn val(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.scip_ptr.raw, self.raw.as_ptr(), var.raw) }
    }

    /// Sets the value of a variable in the solution.
    pub fn set_val(&self, var: &Variable, val: f64) {
        scip_call_panic!(ffi::SCIPsetSolVal(
            self.scip_ptr.raw,
            self.raw.as_ptr(),
            var.raw,
            val
        ));
    }

    /// Returns the solution as a var-name to value map.
    pub fn as_name_map(&self) -> std::collections::HashMap<String, f64> {
        let vars = unsafe { ffi::SCIPgetVars(self.scip_ptr.raw) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.scip_ptr.raw) };
        let mut map = std::collections::HashMap::new();
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr.raw, self.raw.as_ptr(), var) };
            let eps = unsafe { ffi::SCIPepsilon(self.scip_ptr.raw) };
            if val > eps || val < -eps {
                let name_ptr = unsafe { ffi::SCIPvarGetName(var) };
                // from CString
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap() };
                map.insert(name.to_string(), val);
            }
        }
        map
    }

    /// Returns the solution as a var-id to value map.
    pub fn as_id_map(&self) -> std::collections::HashMap<i32, f64> {
        let vars = unsafe { ffi::SCIPgetVars(self.scip_ptr.raw) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.scip_ptr.raw) };
        let mut map = std::collections::HashMap::new();
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr.raw, self.raw.as_ptr(), var) };
            let eps = unsafe { ffi::SCIPepsilon(self.scip_ptr.raw) };
            if val > eps || val < -eps {
                let id = unsafe { ffi::SCIPvarGetProbindex(var) };
                map.insert(id, val);
            }
        }
        map
    }
}

impl fmt::Debug for Solution {
    /// Formats the solution for debugging purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj_val = self.obj_val();
        writeln!(f, "Solution with obj val: {obj_val}")?;
        let vars = unsafe { ffi::SCIPgetOrigVars(self.scip_ptr.raw) };
        let n_vars = unsafe { ffi::SCIPgetNOrigVars(self.scip_ptr.raw) };
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr.raw, self.raw.as_ptr(), var) };
            let eps = unsafe { ffi::SCIPepsilon(self.scip_ptr.raw) };
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
        assert!(!sol.inner().is_null());

        let debug_str = format!("{sol:?}");
        assert!(debug_str.contains("Solution with obj val: 200"));
        assert!(debug_str.contains("Var x1=40"));
        assert!(debug_str.contains("Var x2=20"));

        let vars = model.vars();
        assert_eq!(sol.val(&vars[0]), 40.);
        assert_eq!(sol.val(&vars[1]), 20.);

        assert_eq!(sol.obj_val(), model.obj_val());

        let sol_name_map = sol.as_name_map();
        assert_eq!(sol_name_map.get("t_x1").unwrap(), &40.);
        assert_eq!(sol_name_map.get("t_x2").unwrap(), &20.);

        let sol_id_map = sol.as_id_map();
        assert_eq!(sol_id_map.get(&0).unwrap(), &40.);
        assert_eq!(sol_id_map.get(&1).unwrap(), &20.);
    }
}
