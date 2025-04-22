use crate::scip::ScipPtr;
use crate::{ffi, Row};
use std::rc::Rc;

/// A constraint in an optimization problem.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Constraint {
    /// A pointer to the underlying `SCIP_CONS` C struct.
    pub(crate) raw: *mut ffi::SCIP_CONS,
    /// A reference to the SCIP instance that owns this constraint (to prevent freeing the model while the constraint is live).
    pub(crate) scip: Rc<ScipPtr>,
}

impl Constraint {
    /// Returns a pointer to the underlying `SCIP_CONS` C struct.
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

    /// Returns the dual solution of the linear constraint in the current LP.
    /// Returns `None` if the constraint is not a linear constraint.
    pub fn dual_sol(&self) -> Option<f64> {
        let cons_handler = unsafe { ffi::SCIPconsGetHdlr(self.raw) };
        if cons_handler.is_null() {
            return None;
        }
        let cons_handler_name = unsafe { ffi::SCIPconshdlrGetName(cons_handler) };
        if cons_handler_name.is_null() {
            return None;
        }
        let cons_handler_name = unsafe { std::ffi::CStr::from_ptr(cons_handler_name) };
        if cons_handler_name.to_str().unwrap() != "linear" {
            return None;
        }

        Some(unsafe { ffi::SCIPgetDualsolLinear(self.scip.raw, self.raw) })
    }

    /// Returns the modifiable flag of the constraint
    pub fn is_modifiable(&self) -> bool {
        self.scip.cons_is_modifiable(self)
    }

    /// Returns the removable flag of the constraint
    pub fn is_removable(&self) -> bool {
        self.scip.cons_is_removable(self)
    }

    /// Returns whether the constraint should be separated during LP processing
    pub fn is_separated(&self) -> bool {
        self.scip.cons_is_separated(self)
    }
    
    /// Returns the corresponding transformed constraint.
    /// Returns `None` if the transformed constraint does not exist (yet).
    pub fn transformed(&self) -> Option<Constraint> {
        self.scip
            .get_transformed_cons(self)
            .ok()
            .flatten()
            .map(|raw| Constraint {
                raw,
                scip: self.scip.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{minimal_model, prelude::*};
    use core::f64;

    #[test]
    fn test_constraint_mem_safety() {
        // Create model
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize);

        let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
        let cons = model.add_cons(vec![&x1], &[1.], 4., 4., "cons");
        drop(model);

        assert_eq!(cons.name(), "cons");
    }

    #[test]
    fn test_constraint_transformed_no_transformed() {
        let mut model = minimal_model().hide_output().maximize();
        let x1 = model.add_var(0.0, f64::INFINITY, 10.0, "x1", VarType::Continuous);
        let cons = model.add_cons(vec![&x1], &[1.0], 0.0, 5.0, "cons");

        assert!(model.solve().best_sol().is_some());
        assert!(cons.transformed().is_none());
    }

    #[test]
    fn test_constraint_transformed_with_transformed() {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("prob")
            .maximize();

        let x1 = model.add_var(0.0, f64::INFINITY, 10.0, "x1", VarType::Continuous);
        let cons = model.add_cons(vec![&x1], &[1.0], 0.0, 5.0, "cons");
        model.set_cons_modifiable(&cons, true);

        assert!(model.solve().best_sol().is_some());
        assert!(cons.transformed().is_some());
        let dual = cons.transformed().unwrap().dual_sol().unwrap();
        assert!(dual + 10.0 < f64::EPSILON);
    }
}
