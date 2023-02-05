use crate::c_api;

pub struct Constraint {
    pub(crate) ptr: *mut c_api::SCIP_CONS,
    pub(crate) scip_ptr: *mut c_api::SCIP,
}

impl Constraint {
    pub fn new(scip_ptr: *mut c_api::SCIP, ptr: *mut c_api::SCIP_CONS) -> Self {
        Constraint { scip_ptr, ptr }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = c_api::SCIPconsGetName(self.ptr);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}

impl Drop for Constraint {
    fn drop(&mut self) {
        // release all vars in constraint
        let nvars = unsafe { c_api::SCIPgetNVarsLinear(self.scip_ptr, self.ptr) };
        let vars = unsafe { c_api::SCIPgetVarsLinear(self.scip_ptr, self.ptr) };
        for i in 0..nvars {
            let mut var = unsafe { *vars.offset(i as isize) };
            unsafe { c_api::SCIPreleaseVar(self.scip_ptr, &mut var) };
        }
    }
}

