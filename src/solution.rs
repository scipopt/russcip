use std::fmt;

use crate::ffi;
use crate::variable::Variable;

/// A wrapper for a SCIP solution.
pub struct Solution {
    pub(crate) scip_ptr: *mut ffi::SCIP,
    pub(crate) raw: *mut ffi::SCIP_SOL,
}

impl Solution {
    /// Returns the objective value of the solution.
    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.scip_ptr, self.raw) }
    }

    /// Returns the value of a variable in the solution.
    pub fn get_var_val(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var.raw) }
    }
}

impl fmt::Debug for Solution {
    /// Formats the solution for debugging purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj_val = self.get_obj_val();
        writeln!(f, "Solution with obj val: {obj_val}")?;
        let vars = unsafe { ffi::SCIPgetVars(self.scip_ptr) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.scip_ptr) };
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var) };
            if val > 0.0 {
                let name_ptr = unsafe { ffi::SCIPvarGetName(var) };
                // from CString
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap() };
                writeln!(f, "Var {name}={val}")?;
            }
        }
        Ok(())
    }
}
