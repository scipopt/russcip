use crate::c_api;

pub struct Variable {
    pub scip_var: *mut c_api::SCIP_VAR,
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
