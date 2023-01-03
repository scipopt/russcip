#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, mem::MaybeUninit};
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! scip_call {
    ($res:expr) => {
        let res = unsafe { $res };
        if res != SCIP_Retcode_SCIP_OKAY {
            return Err(res);
        }
    };
}

pub struct Model {
    scip: *mut SCIP,
}

impl Model {
    pub fn new() -> Result<Self, SCIP_RETCODE> {
        let mut model: *mut SCIP = unsafe { MaybeUninit::uninit().assume_init() };
        scip_call!(SCIPcreate(&mut model));
        Ok(Model { scip: model })
    }
    pub fn include_default_plugins(&mut self) -> Result<(), SCIP_RETCODE> {
        scip_call! { SCIPincludeDefaultPlugins(self.scip)};
        Ok(())
    }
    pub fn read_prob(&mut self, filename: &str) -> Result<(), SCIP_RETCODE> {
        let filename = CString::new(filename).unwrap();
        scip_call! { SCIPreadProb(self.scip, filename.as_ptr(), std::ptr::null_mut()) };
        Ok(())
    }
    pub fn solve(&mut self) -> Result<(), SCIP_RETCODE> {
        scip_call! { SCIPsolve(self.scip) };
        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe { SCIPfree(&mut self.scip) };
    }
}

pub fn solve(problem_path: String) {
    let mut model = Model::new().unwrap();
    model.include_default_plugins().unwrap();
    model.read_prob(&problem_path).unwrap();
    model.solve().unwrap();
}
