#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, mem::MaybeUninit};
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
    pub fn get_status(&mut self) -> SCIPStatus {
        let status = unsafe { c_api::SCIPgetStatus(self.scip) };
        SCIPStatus::from_c_scip_status(status).unwrap()
    }

    pub fn get_obj_val(&mut self) -> f64 {
        unsafe { c_api::SCIPgetPrimalbound(self.scip) }
    }

    pub fn get_n_vars(&mut self) -> usize {
        unsafe { c_api::SCIPgetNVars(self.scip) as usize }
    }

    pub fn print_version(&self) -> Result<(), SCIPRetcode> {
        unsafe { c_api::SCIPprintVersion(self.scip, std::ptr::null_mut()) };
        Ok(())
    }
}


impl Drop for Model {
    fn drop(&mut self) {
        unsafe { c_api::SCIPfree(&mut self.scip) };
    }
}

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

pub fn solve(problem_path: String) {
    let mut model = Model::new().unwrap();
    model.include_default_plugins().unwrap();
    model.read_prob(&problem_path).unwrap();
    model.solve().unwrap();
}
