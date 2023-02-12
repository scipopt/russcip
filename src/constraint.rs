use crate::ffi;

pub struct Constraint {
    pub(crate) ptr: *mut ffi::SCIP_CONS,
    pub(crate) scip_ptr: *mut ffi::SCIP,
}

impl Constraint {
    pub fn new(scip_ptr: *mut ffi::SCIP, ptr: *mut ffi::SCIP_CONS) -> Self {
        Constraint { scip_ptr, ptr }
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = ffi::SCIPconsGetName(self.ptr);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}

impl Drop for Constraint {
    fn drop(&mut self) {
        unsafe { ffi::SCIPreleaseCons(self.scip_ptr, &mut self.ptr) };
    }
}

