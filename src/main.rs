#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    unsafe {
        let mut model: *mut SCIP = std::mem::uninitialized();
        let x = SCIPcreate(&mut model);
        SCIPprintVersion(model, std::ptr::null_mut());
        SCIPfree(&mut model);
    }
}
