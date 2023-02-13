use std::fmt;

use crate::ffi;
use crate::model::Model;
use crate::variable::Variable;

pub struct Solution<'a> {
    pub(crate) model: &'a Model,
    pub(crate) raw: *mut ffi::SCIP_SOL,
}

impl<'a> Solution<'a> {
    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.model.scip, self.raw) }
    }

    pub fn get_var_val(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.model.scip, self.raw, var.raw) }
    }
}

impl<'a> fmt::Debug for Solution<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj_val = self.get_obj_val();
        write!(f, "Solution with obj val: {}\n", obj_val)?;
        let vars = unsafe { ffi::SCIPgetVars(self.model.scip) };
        let n_vars = unsafe { ffi::SCIPgetNVars(self.model.scip) };
        for i in 0..n_vars {
            let var = unsafe { *vars.offset(i as isize) };
            let val = unsafe { ffi::SCIPgetSolVal(self.model.scip, self.raw, var) };
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