use crate::ffi;
use crate::model::Model;
use crate::variable::Variable;
use crate::retcode::Retcode;
use std::fmt;
use std::rc::Rc;

pub struct Solution<'a> {
    model: Rc<&'a Model>,
    ptr: *mut ffi::SCIP_SOL,
}

impl<'a> Solution<'a> {
    pub fn new(
        scip_ptr: Rc<&'a Model>,
        scip_sol_prt: *mut ffi::SCIP_Sol,
    ) -> Result<Self, Retcode> {
        Ok(Solution {
            model: scip_ptr,
            ptr: scip_sol_prt,
        })
    }

    pub fn get_obj_val(&self) -> f64 {
        unsafe { ffi::SCIPgetSolOrigObj(self.model.scip, self.ptr) }
    }

    pub fn get_var_val(&self, var: &Variable) -> f64 {
        unsafe { ffi::SCIPgetSolVal(self.model.scip, self.ptr, var.ptr) }
    }
}

impl<'a> fmt::Debug for Solution<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj_val = self.get_obj_val();
        write!(f, "Solution with obj val: {}\n", obj_val)?;
        for var in self.model.get_vars() {
            let val = self.get_var_val(&var);
            if val > 0.0 {
                write!(f, "Var {}={}\n", var.get_name(), val)?;
            }
        }
        Ok(())
    }
}