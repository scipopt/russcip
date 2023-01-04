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
            return Err(SCIPRetcode::from_i8(res as i8).unwrap());
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
    pub fn from_i8(val: i8) -> Option<Self> {
        match val {
            1 => Some(SCIPRetcode::OKAY),
            0 => Some(SCIPRetcode::ERROR),
            -1 => Some(SCIPRetcode::NOMEMORY),
            -2 => Some(SCIPRetcode::READERROR),
            -3 => Some(SCIPRetcode::WRITEERROR),
            -4 => Some(SCIPRetcode::NOFILE),
            -5 => Some(SCIPRetcode::FILECREATEERROR),
            -6 => Some(SCIPRetcode::LPERROR),
            -7 => Some(SCIPRetcode::NOPROBLEM),
            -8 => Some(SCIPRetcode::INVALIDCALL),
            -9 => Some(SCIPRetcode::INVALIDDATA),
            -10 => Some(SCIPRetcode::INVALIDRESULT),
            -11 => Some(SCIPRetcode::PLUGINNOTFOUND),
            -12 => Some(SCIPRetcode::PARAMETERUNKNOWN),
            -13 => Some(SCIPRetcode::PARAMETERWRONGTYPE),
            -14 => Some(SCIPRetcode::PARAMETERWRONGVAL),
            -15 => Some(SCIPRetcode::KEYALREADYEXISTING),
            -16 => Some(SCIPRetcode::MAXDEPTHLEVEL),
            -17 => Some(SCIPRetcode::BRANCHERROR),
            -18 => Some(SCIPRetcode::NOTIMPLEMENTED),
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
    pub fn from_i8(val: i8) -> Option<Self> {
        match val {
            0 => Some(SCIPStatus::UNKNOWN),
            1 => Some(SCIPStatus::USERINTERRUPT),
            2 => Some(SCIPStatus::NODELIMIT),
            3 => Some(SCIPStatus::TOTALNODELIMIT),
            4 => Some(SCIPStatus::STALLNODELIMIT),
            5 => Some(SCIPStatus::TIMELIMIT),
            6 => Some(SCIPStatus::MEMLIMIT),
            7 => Some(SCIPStatus::GAPLIMIT),
            8 => Some(SCIPStatus::SOLLIMIT),
            9 => Some(SCIPStatus::BESTSOLLIMIT),
            10 => Some(SCIPStatus::RESTARTLIMIT),
            11 => Some(SCIPStatus::OPTIMAL),
            12 => Some(SCIPStatus::INFEASIBLE),
            13 => Some(SCIPStatus::UNBOUNDED),
            14 => Some(SCIPStatus::INFORUNBD),
            15 => Some(SCIPStatus::TERMINATE),
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
