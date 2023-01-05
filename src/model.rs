use std::mem::MaybeUninit;

use crate::c_api;
use crate::retcode::SCIPRetcode;
use crate::status::SCIPStatus;
use std::ffi::CString;
use std::rc::Rc;
use crate::variable::Variable;
use crate::solution::Solution;
use crate::scip_call;

pub struct Model {
    pub scip: *mut c_api::SCIP,
}

impl Model {
    pub fn new() -> Result<Self, SCIPRetcode> {
        let mut model: *mut c_api::SCIP = unsafe { MaybeUninit::uninit().assume_init() };
        scip_call!(c_api::SCIPcreate(&mut model));
        Ok(Model { scip: model })
    }

    pub fn include_default_plugins(&mut self) -> Result<(), SCIPRetcode> {
        scip_call! { c_api::SCIPincludeDefaultPlugins(self.scip)};
        Ok(())
    }

    pub fn read_prob(&mut self, filename: &str) -> Result<(), SCIPRetcode> {
        let filename = CString::new(filename).unwrap();
        scip_call! { c_api::SCIPreadProb(self.scip, filename.as_ptr(), std::ptr::null_mut()) };
        Ok(())
    }
    pub fn solve(&mut self) -> Result<(), SCIPRetcode> {
        scip_call! { c_api::SCIPsolve(self.scip) };
        Ok(())
    }
    pub fn get_status(&self) -> SCIPStatus {
        let status = unsafe { c_api::SCIPgetStatus(self.scip) };
        SCIPStatus::from_c_scip_status(status).unwrap()
    }

    pub fn get_obj_val(&self) -> f64 {
        unsafe { c_api::SCIPgetPrimalbound(self.scip) }
    }

    pub fn get_n_vars(&self) -> usize {
        unsafe { c_api::SCIPgetNVars(self.scip) as usize }
    }

    pub fn print_version(&self) -> Result<(), SCIPRetcode> {
        unsafe { c_api::SCIPprintVersion(self.scip, std::ptr::null_mut()) };
        Ok(())
    }

    pub fn get_best_sol(&self) -> Result<Solution, SCIPRetcode> {
        let sol = unsafe { c_api::SCIPgetBestSol(self.scip) };
        let sol = Solution::new(Rc::new(self), sol)?;
        Ok(sol)
    }

    pub fn get_vars(&self) -> Vec<Variable> {
        let n_vars = self.get_n_vars();
        let mut vars = Vec::with_capacity(n_vars);
        let scip_vars = unsafe { c_api::SCIPgetVars(self.scip) };
        for i in 0..n_vars {
            let scip_var = unsafe { *scip_vars.offset(i as isize) };
            vars.push(Variable::new(scip_var));
        }
        vars
    }

    pub fn set_str_param(&mut self, param: &str, value: &str) -> Result<(), SCIPRetcode> {
        let param = CString::new(param).unwrap();
        let value = CString::new(value).unwrap();
        scip_call! { c_api::SCIPsetStringParam(self.scip, param.as_ptr(), value.as_ptr()) };
        Ok(())
    }

    pub fn set_int_param(&mut self, param: &str, value: i32) -> Result<(), SCIPRetcode> {
        let param = CString::new(param).unwrap();
        scip_call! { c_api::SCIPsetIntParam(self.scip, param.as_ptr(), value) };
        Ok(())
    }

    pub fn set_real_param(&mut self, param: &str, value: f64) -> Result<(), SCIPRetcode> {
        let param = CString::new(param).unwrap();
        scip_call! { c_api::SCIPsetRealParam(self.scip, param.as_ptr(), value) };
        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe { c_api::SCIPfree(&mut self.scip) };
    }
}