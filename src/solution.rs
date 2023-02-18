use std::fmt;

use crate::ffi;
use crate::variable::Variable;

pub struct Solution {
    pub(crate) scip_ptr: *mut ffi::SCIP,
    pub(crate) raw: *mut ffi::SCIP_SOL,
}

impl Solution {
    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.scip_ptr, self.raw) }
    }

    pub fn get_var_val(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var.raw) }
    }
}

impl fmt::Debug for Solution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj_val = self.get_obj_val();
        write!(f, "Solution with obj val: {}\n", obj_val)?;
        let vars = unsafe { ffi::SCIPgetVars(self.scip_ptr) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.scip_ptr) };
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.scip_ptr, self.raw, var) };
            if val > 0.0 {
                let name_ptr = unsafe { ffi::SCIPvarGetName(var) };
                // from CString
                let name = unsafe { std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap() };
                write!(f, "Var {}={}\n", name, val)?;
            }
        }
        Ok(())
    }
}
