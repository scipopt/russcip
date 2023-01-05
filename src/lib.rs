#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, fmt, mem::MaybeUninit, rc::Rc};
mod c_api {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

macro_rules! scip_call {
    ($res:expr) => {
        let res = unsafe { $res };
        if res != c_api::SCIP_Retcode_SCIP_OKAY {
            return Err(SCIPRetcode::from_c_scip_retcode(res).unwrap());
        }
    };
}

pub struct Model {
    scip: *mut c_api::SCIP,
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

pub struct Solution<'a> {
    model: Rc<&'a Model>,
    scip_sol: *mut c_api::SCIP_SOL,
}

impl<'a> Solution<'a> {
    pub fn new(
        scip_ptr: Rc<&'a Model>,
        scip_sol_prt: *mut c_api::SCIP_Sol,
    ) -> Result<Self, SCIPRetcode> {
        Ok(Solution {
            model: scip_ptr,
            scip_sol: scip_sol_prt,
        })
    }

    pub fn get_obj_val(&self) -> f64 {
        unsafe { c_api::SCIPgetSolOrigObj(self.model.scip, self.scip_sol) }
    }

    pub fn get_var_val(&self, var: &Variable) -> f64 {
        unsafe { c_api::SCIPgetSolVal(self.model.scip, self.scip_sol, var.scip_var) }
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

pub struct Variable {
    scip_var: *mut c_api::SCIP_VAR,
}

impl Variable {
    pub fn new(scip_var: *mut c_api::SCIP_VAR) -> Self {
        Variable { scip_var }
    }

    pub fn get_name(&self) -> String {
        let name = unsafe { c_api::SCIPvarGetName(self.scip_var) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    pub fn get_obj(&self) -> f64 {
        unsafe { c_api::SCIPvarGetObj(self.scip_var) }
    }

    pub fn get_lb(&self) -> f64 {
        unsafe { c_api::SCIPvarGetLbLocal(self.scip_var) }
    }

    pub fn get_ub(&self) -> f64 {
        unsafe { c_api::SCIPvarGetUbLocal(self.scip_var) }
    }
}

// TODO: implement parameter overloading for variable to use SCIP's tolerance values

#[derive(Debug)]
pub enum SCIPRetcode {
    OKAY,
    ERROR,
    NOMEMORY,
    READERROR,
    WRITEERROR,
    NOFILE,
    FILECREATEERROR,
    LPERROR,
    NOPROBLEM,
    INVALIDCALL,
    INVALIDDATA,
    INVALIDRESULT,
    PLUGINNOTFOUND,
    PARAMETERUNKNOWN,
    PARAMETERWRONGTYPE,
    PARAMETERWRONGVAL,
    KEYALREADYEXISTING,
    MAXDEPTHLEVEL,
    BRANCHERROR,
    NOTIMPLEMENTED,
}

impl SCIPRetcode {
    pub fn from_c_scip_retcode(val: c_api::SCIP_Retcode) -> Option<Self> {
        match val {
            c_api::SCIP_Retcode_SCIP_OKAY => Some(SCIPRetcode::OKAY),
            c_api::SCIP_Retcode_SCIP_ERROR => Some(SCIPRetcode::ERROR),
            c_api::SCIP_Retcode_SCIP_NOMEMORY => Some(SCIPRetcode::NOMEMORY),
            c_api::SCIP_Retcode_SCIP_READERROR => Some(SCIPRetcode::READERROR),
            c_api::SCIP_Retcode_SCIP_WRITEERROR => Some(SCIPRetcode::WRITEERROR),
            c_api::SCIP_Retcode_SCIP_NOFILE => Some(SCIPRetcode::NOFILE),
            c_api::SCIP_Retcode_SCIP_FILECREATEERROR => Some(SCIPRetcode::FILECREATEERROR),
            c_api::SCIP_Retcode_SCIP_LPERROR => Some(SCIPRetcode::LPERROR),
            c_api::SCIP_Retcode_SCIP_NOPROBLEM => Some(SCIPRetcode::NOPROBLEM),
            c_api::SCIP_Retcode_SCIP_INVALIDCALL => Some(SCIPRetcode::INVALIDCALL),
            c_api::SCIP_Retcode_SCIP_INVALIDDATA => Some(SCIPRetcode::INVALIDDATA),
            c_api::SCIP_Retcode_SCIP_INVALIDRESULT => Some(SCIPRetcode::INVALIDRESULT),
            c_api::SCIP_Retcode_SCIP_PLUGINNOTFOUND => Some(SCIPRetcode::PLUGINNOTFOUND),
            c_api::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => Some(SCIPRetcode::PARAMETERUNKNOWN),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => Some(SCIPRetcode::PARAMETERWRONGTYPE),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => Some(SCIPRetcode::PARAMETERWRONGVAL),
            c_api::SCIP_Retcode_SCIP_KEYALREADYEXISTING => Some(SCIPRetcode::KEYALREADYEXISTING),
            c_api::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => Some(SCIPRetcode::MAXDEPTHLEVEL),
            c_api::SCIP_Retcode_SCIP_BRANCHERROR => Some(SCIPRetcode::BRANCHERROR),
            c_api::SCIP_Retcode_SCIP_NOTIMPLEMENTED => Some(SCIPRetcode::NOTIMPLEMENTED),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SCIPStatus {
    UNKNOWN,
    USERINTERRUPT,
    NODELIMIT,
    TOTALNODELIMIT,
    STALLNODELIMIT,
    TIMELIMIT,
    MEMLIMIT,
    GAPLIMIT,
    SOLLIMIT,
    BESTSOLLIMIT,
    RESTARTLIMIT,
    OPTIMAL,
    INFEASIBLE,
    UNBOUNDED,
    INFORUNBD,
    TERMINATE,
}

impl SCIPStatus {
    pub fn from_c_scip_status(val: c_api::SCIP_Status) -> Option<Self> {
        match val {
            c_api::SCIP_Status_SCIP_STATUS_UNKNOWN => Some(SCIPStatus::UNKNOWN),
            c_api::SCIP_Status_SCIP_STATUS_USERINTERRUPT => Some(SCIPStatus::USERINTERRUPT),
            c_api::SCIP_Status_SCIP_STATUS_NODELIMIT => Some(SCIPStatus::NODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT => Some(SCIPStatus::TOTALNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_STALLNODELIMIT => Some(SCIPStatus::STALLNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TIMELIMIT => Some(SCIPStatus::TIMELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_MEMLIMIT => Some(SCIPStatus::MEMLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_GAPLIMIT => Some(SCIPStatus::GAPLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_SOLLIMIT => Some(SCIPStatus::SOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT => Some(SCIPStatus::BESTSOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_RESTARTLIMIT => Some(SCIPStatus::RESTARTLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_OPTIMAL => Some(SCIPStatus::OPTIMAL),
            c_api::SCIP_Status_SCIP_STATUS_INFEASIBLE => Some(SCIPStatus::INFEASIBLE),
            c_api::SCIP_Status_SCIP_STATUS_UNBOUNDED => Some(SCIPStatus::UNBOUNDED),
            c_api::SCIP_Status_SCIP_STATUS_INFORUNBD => Some(SCIPStatus::INFORUNBD),
            c_api::SCIP_Status_SCIP_STATUS_TERMINATE => Some(SCIPStatus::TERMINATE),
            _ => None,
        }
    }
}
