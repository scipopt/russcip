use crate::{ffi, model::Model};

pub struct Constraint<'a> {
    pub(crate) model: &'a Model,
    pub(crate) raw: *mut ffi::SCIP_CONS,
}

impl<'a> Constraint<'a> {
    pub fn get_name(&self) -> String {
        unsafe {
            let name = ffi::SCIPconsGetName(self.raw);
            String::from(std::ffi::CStr::from_ptr(name).to_str().unwrap())
        }
    }
}

impl<'a> Drop for Constraint<'a> {
    fn drop(&mut self) {
        unsafe { ffi::SCIPreleaseCons(self.model.scip, &mut self.raw) };
    }
}

