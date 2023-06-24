use crate::ffi;

/// A constraint in an optimization problem.
#[derive(Debug)]
pub struct Constraint {
    /// A pointer to the underlying `SCIP_CONS` C struct.
    pub(crate) raw: *mut ffi::SCIP_CONS,
}

impl Constraint {
    /// Returns a pointer to the underlying `SCIP_CONS` C struct.
    #[cfg(feature = "raw")]
    pub fn inner(&self) -> *mut ffi::SCIP_CONS {
        self.raw
    }

    /// Returns the name of the constraint.
    pub fn name(&self) -> String {
        unsafe {
            let name = ffi::SCIPconsGetName(self.raw);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}
