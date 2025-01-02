use crate::{ffi, Row};
use crate::scip::ScipPtr;
use std::rc::Rc;

/// A constraint in an optimization problem.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Constraint {
    /// A pointer to the underlying `SCIP_CONS` C struct.
    pub(crate) raw: *mut ffi::SCIP_CONS,
    /// A reference to the SCIP instance that owns this constraint (to prevent freeing the model while the constraint is live).
    pub(crate) scip: Rc<ScipPtr>,
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

    /// Returns the row associated with the constraint.
    pub fn row(&self) -> Option<Row> {
        let row_ptr = unsafe { ffi::SCIPconsGetRow(self.scip.raw, self.raw) };
        if row_ptr.is_null() {
            None
        } else {
            Some(Row {
                raw: row_ptr,
                scip: Rc::clone(&self.scip),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_constraint_mem_safety() {
        // Create model
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let cons = model.add_cons(vec![x1], &[1.], 4., 4., "cons");
        drop(model);

        assert_eq!(cons.name(), "cons");
    }
}
