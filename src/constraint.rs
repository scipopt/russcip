use crate::ffi;

pub struct Constraint {
    pub(crate) raw: *mut ffi::SCIP_CONS,
    pub(crate) scip_ptr: *mut ffi::SCIP,
}

impl Constraint {
    pub fn get_name(&self) -> String {
        unsafe {
            let name = ffi::SCIPconsGetName(self.raw);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}