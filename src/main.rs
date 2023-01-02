#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CString, mem::MaybeUninit};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


fn solve(problem_path: String) {
    unsafe {
        let mut model: *mut SCIP = MaybeUninit::uninit().assume_init();
        SCIPcreate(&mut model);
        SCIPprintVersion(model, std::ptr::null_mut());
        SCIPincludeDefaultPlugins(model);
        let filename = CString::new(problem_path).unwrap();
        SCIPreadProb(model, filename.as_ptr(), std::ptr::null_mut());
        SCIPsolve(model);
        SCIPfree(&mut model);
    }
}

fn main() {
        let filename = std::env::args().nth(1).expect("Expected filename of lp file as an argument");
        solve(filename) 
}
