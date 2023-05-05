use crate::ffi;

pub struct Constraint {
    pub(crate) raw: *mut ffi::SCIP_CONS,
}

impl Constraint {
    #[cfg(feature = "raw")]
    pub fn inner(&self) -> *mut ffi::SCIP_CONS {
        self.raw
    }

    pub fn get_name(&self) -> String {
        unsafe {
            let name = ffi::SCIPconsGetName(self.raw);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}
