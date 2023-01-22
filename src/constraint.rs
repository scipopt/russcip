use crate::c_api;

pub struct Constraint {
    pub(crate) ptr: *mut c_api::SCIP_CONS,
}

impl Constraint {
    pub fn new(ptr: *mut c_api::SCIP_CONS) -> Self {
        Constraint { ptr }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = c_api::SCIPconsGetName(self.ptr);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}

