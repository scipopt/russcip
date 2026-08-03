use crate::scip::ScipPtr;
use crate::{Row, ffi};
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

/// A non-owning reference to a [`Constraint`].
///
/// A `Constraint` holds a strong `Rc` to the model, which keeps SCIP alive for
/// as long as the handle exists. That is what you want for a local binding, but
/// it forms a reference cycle if the handle is stored somewhere the model itself
/// owns — a plugin, or the datastore — and then the model is never freed.
/// A `ConsRef` holds the model weakly, so it can be stored in those places.
///
/// See [`Constraint::downgrade`].
#[derive(Debug, Clone)]
pub struct ConsRef {
    raw: *mut ffi::SCIP_CONS,
    scip: std::rc::Weak<ScipPtr>,
}

impl ConsRef {
    /// Recovers the [`Constraint`], or `None` if the model has been dropped.
    pub fn upgrade(&self) -> Option<Constraint> {
        self.scip.upgrade().map(|scip| Constraint {
            raw: self.raw,
            scip,
        })
    }
}

impl Constraint {
    /// Returns a pointer to the underlying `SCIP_CONS` C struct.
    pub fn inner(&self) -> *mut ffi::SCIP_CONS {
        self.raw
    }

    /// Produces a non-owning [`ConsRef`], safe to store inside a plugin or the
    /// model's datastore without leaking the model.
    pub fn downgrade(&self) -> ConsRef {
        ConsRef {
            raw: self.raw,
            scip: Rc::downgrade(&self.scip),
        }
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

    /// Returns the Farkas dual solution of the linear constraint in the current (infeasible) LP.
    /// Returns `None` if the constraint is not a linear constraint.
    pub fn farkas_dual_sol(&self) -> Option<f64> {
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

        Some(unsafe { ffi::SCIPgetDualfarkasLinear(self.scip.raw, self.raw) })
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

#[cfg(all(test, feature = "datastore"))]
mod weak_ref_tests {
    use crate::prelude::*;
    use crate::{ConsRef, Constraint};
    use std::cell::Cell;
    use std::rc::Rc;

    struct DropProbe(Rc<Cell<bool>>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }

    /// The model's datastore lives inside SCIP, so a `Constraint` put there
    /// forms a cycle: store -> Constraint -> Rc<ScipPtr> -> SCIP -> store. The
    /// anymap is freed by `ScipPtr::drop`, which is exactly what the cycle
    /// prevents from ever running. A `ConsRef` avoids it.
    #[test]
    fn cons_ref_does_not_keep_the_model_alive() {
        struct Strong {
            _probe: DropProbe,
            _conss: Vec<Constraint>,
        }
        struct Weak {
            _probe: DropProbe,
            conss: Vec<ConsRef>,
        }

        let dropped = Rc::new(Cell::new(false));
        {
            let mut model = Model::default().hide_output();
            let x = model.add(var().name("x").cont(0.0..=1.0));
            let c = model.add(cons().name("c").coef(&x, 1.0).le(1.0));
            model.set_data(Strong {
                _probe: DropProbe(Rc::clone(&dropped)),
                _conss: vec![c],
            });
        }
        assert!(!dropped.get(), "a stored Constraint should leak the model");

        let dropped = Rc::new(Cell::new(false));
        {
            let mut model = Model::default().hide_output();
            let x = model.add(var().name("x").cont(0.0..=1.0));
            let c = model.add(cons().name("c").coef(&x, 1.0).le(1.0));
            model.set_data(Weak {
                _probe: DropProbe(Rc::clone(&dropped)),
                conss: vec![c.downgrade()],
            });
            let stored = &model.get_data::<Weak>().unwrap().conss[0];
            assert_eq!(stored.upgrade().expect("model is alive").name(), "c");
        }
        assert!(dropped.get(), "a stored ConsRef should not");
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
