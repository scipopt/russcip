use crate::scip::ScipPtr;
use crate::{Col, Constraint, Variable, ffi};
use std::ffi::c_int;
use std::rc::Rc;

/// A row in the LP relaxation.
#[derive(Debug, Clone)]
pub struct Row {
    pub(crate) raw: *mut ffi::SCIP_ROW,
    pub(crate) scip: Rc<ScipPtr>,
}

impl Row {
    /// Returns a raw pointer to the underlying `ffi::SCIP_ROW` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_ROW {
        self.raw
    }

    /// Returns the number of non-zero entries in the row.
    pub fn n_non_zeroes(&self) -> usize {
        let len = unsafe { ffi::SCIProwGetNNonz(self.raw) };
        assert!(len >= 0);
        len as usize
    }

    /// Returns the columns of the row.
    pub fn cols(&self) -> Vec<Col> {
        let mut columns = Vec::new();
        let cols_ptr = unsafe { ffi::SCIProwGetCols(self.raw) };
        for i in 0..self.n_non_zeroes() {
            let col_ptr = unsafe { *cols_ptr.add(i) };
            let col = Col {
                raw: col_ptr,
                scip: Rc::clone(&self.scip),
            };
            columns.push(col);
        }
        columns
    }

    /// Returns the index of the row.
    pub fn index(&self) -> usize {
        let id = unsafe { ffi::SCIProwGetIndex(self.raw) };
        assert!(id >= 0);
        id as usize
    }

    /// Returns the left-hand side of the row.
    pub fn lhs(&self) -> f64 {
        unsafe { ffi::SCIProwGetLhs(self.raw) }
    }

    /// Returns the right-hand side of the row.
    pub fn rhs(&self) -> f64 {
        unsafe { ffi::SCIProwGetRhs(self.raw) }
    }

    /// Returns the dual value of the row.
    pub fn dual(&self) -> f64 {
        unsafe { ffi::SCIProwGetDualsol(self.raw) }
    }

    /// Returns the farkas dual value of the row.
    pub fn farkas_dual(&self) -> f64 {
        unsafe { ffi::SCIProwGetDualfarkas(self.raw) }
    }

    /// Returns the basis status of the row.
    pub fn basis_status(&self) -> BasisStatus {
        let status = unsafe { ffi::SCIProwGetBasisStatus(self.raw) };
        status.into()
    }

    /// Returns the name of the row.
    pub fn name(&self) -> String {
        let name = unsafe { ffi::SCIProwGetName(self.raw) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    /// Returns the age of the row.
    pub fn age(&self) -> usize {
        let age = unsafe { ffi::SCIProwGetAge(self.raw) };
        assert!(age >= 0);
        age as usize
    }

    /// Returns the rank of the row.
    pub fn rank(&self) -> usize {
        let rank = unsafe { ffi::SCIProwGetRank(self.raw) };
        assert!(rank >= 0);
        rank as usize
    }

    /// Returns whether the row is local.
    pub fn is_local(&self) -> bool {
        (unsafe { ffi::SCIProwIsLocal(self.raw) }) != 0
    }

    /// Returns whether the row is modifiable.
    pub fn is_modifiable(&self) -> bool {
        (unsafe { ffi::SCIProwIsModifiable(self.raw) }) != 0
    }

    /// Returns whether the row is removable.
    pub fn is_removable(&self) -> bool {
        (unsafe { ffi::SCIProwIsRemovable(self.raw) }) != 0
    }

    /// Returns whether the row is integral; the activity of an integral row (without the constant) is always integral.
    pub fn is_integral(&self) -> bool {
        (unsafe { ffi::SCIProwIsIntegral(self.raw) }) != 0
    }

    /// Returns the origin type of the row.
    pub fn origin_type(&self) -> RowOrigin {
        let origin = unsafe { ffi::SCIProwGetOrigintype(self.raw) };
        origin.into()
    }

    /// Returns the constraint associated with the row (if it was created by a constraint).
    pub fn constraint(&self) -> Option<Constraint> {
        let cons_ptr = unsafe { ffi::SCIProwGetOriginCons(self.raw) };
        if cons_ptr.is_null() {
            None
        } else {
            let cons = Constraint {
                raw: cons_ptr,
                scip: Rc::clone(&self.scip),
            };
            Some(cons)
        }
    }

    /// Returns whether the row is in the global cut pool.
    pub fn is_in_global_cut_pool(&self) -> bool {
        (unsafe { ffi::SCIProwIsInGlobalCutpool(self.raw) }) != 0
    }

    /// Returns whether the row is in the current LP.
    pub fn is_in_lp(&self) -> bool {
        (unsafe { ffi::SCIProwIsInLP(self.raw) }) != 0
    }

    /// Returns whether the position of the row in the current LP.
    pub fn lp_position(&self) -> Option<usize> {
        if self.is_in_lp() {
            let pos = unsafe { ffi::SCIProwGetLPPos(self.raw) };
            Some(pos as usize)
        } else {
            None
        }
    }

    /// Returns the depth of the row; the depth in the tree when the row was introduced.
    pub fn depth(&self) -> usize {
        let depth = unsafe { ffi::SCIProwGetLPDepth(self.raw) };
        assert!(depth >= 0);
        depth as usize
    }

    /// Returns the number of times that this row has been sharp in an optimal LP solution.
    pub fn active_lp_count(&self) -> usize {
        let count = unsafe { ffi::SCIProwGetActiveLPCount(self.raw) };
        assert!(count >= 0);
        count as usize
    }

    /// Returns the number of LPs since this row has been created.
    pub fn n_lp_since_create(&self) -> usize {
        let count = unsafe { ffi::SCIProwGetNLPsAfterCreation(self.raw) };
        assert!(count >= 0);
        count as usize
    }

    /// Sets the rank of the row.
    pub fn set_rank(&mut self, rank: usize) {
        unsafe { ffi::SCIProwChgRank(self.raw, rank as c_int) };
    }

    /// Sets the coefficient of a variable in the row.
    pub fn set_coeff(&mut self, var: &Variable, coeff: f64) {
        unsafe { ffi::SCIPaddVarToRow(self.scip.raw, self.raw, var.raw, coeff) };
    }
}

impl PartialEq for Row {
    fn eq(&self, other: &Self) -> bool {
        self.index() == other.index() && self.raw == other.raw
    }
}

/// The basis status of a row.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BasisStatus {
    /// The row is at its lower bound.
    Lower,
    /// The row is basic.
    Basic,
    /// The row is at its upper bound.
    Upper,
    /// The row is at zero.
    Zero,
}

impl From<ffi::SCIP_BASESTAT> for BasisStatus {
    fn from(status: ffi::SCIP_BASESTAT) -> Self {
        match status {
            ffi::SCIP_BaseStat_SCIP_BASESTAT_LOWER => BasisStatus::Lower,
            ffi::SCIP_BaseStat_SCIP_BASESTAT_BASIC => BasisStatus::Basic,
            ffi::SCIP_BaseStat_SCIP_BASESTAT_UPPER => BasisStatus::Upper,
            ffi::SCIP_BaseStat_SCIP_BASESTAT_ZERO => BasisStatus::Zero,
            _ => panic!("Unknown basis status"),
        }
    }
}

/// The origin type of row.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowOrigin {
    /// The row was created by a constraint handler.
    ConsHandler,
    /// The row was created by a constraint.
    Constraint,
    /// The row was created by reoptimization.
    Reoptimization,
    /// The row was created by a separator.
    Separator,
    /// The origin is unspecified.
    Unspecified,
}

impl From<ffi::SCIP_ROWORIGINTYPE> for RowOrigin {
    fn from(origin: ffi::SCIP_ROWORIGINTYPE) -> Self {
        match origin {
            ffi::SCIP_RowOriginType_SCIP_ROWORIGINTYPE_CONSHDLR => RowOrigin::ConsHandler,
            ffi::SCIP_RowOriginType_SCIP_ROWORIGINTYPE_CONS => RowOrigin::Constraint,
            ffi::SCIP_RowOriginType_SCIP_ROWORIGINTYPE_REOPT => RowOrigin::Reoptimization,
            ffi::SCIP_RowOriginType_SCIP_ROWORIGINTYPE_SEPA => RowOrigin::Separator,
            ffi::SCIP_RowOriginType_SCIP_ROWORIGINTYPE_UNSPEC => RowOrigin::Unspecified,
            _ => panic!("Unknown row origin type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::eventhdlr;
    use crate::{Event, ProblemOrSolving};
    use crate::{
        EventMask, Eventhdlr, Model, ModelWithProblem, ObjSense, ParamSetting, Solving, VarType,
    };
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_row() {
        struct RowTesterEventHandler {
            /// Set once the row assertions run, so the test cannot pass vacuously
            /// if the inspected row is never in the LP.
            checked: Rc<AtomicBool>,
        }

        impl Eventhdlr for RowTesterEventHandler {
            fn get_type(&self) -> EventMask {
                EventMask::FIRST_LP_SOLVED
            }

            fn execute(
                &mut self,
                model: Model<Solving>,
                _eventhdlr: crate::SCIPEventhdlr,
                _event: Event,
            ) {
                // Since SCIP 10 this event also fires for the initial (empty) LP,
                // where the constraint has no LP row yet; only inspect once it does.
                let first_cons = model.conss()[0].clone();
                let Some(mut row) = first_cons.row() else {
                    return;
                };
                let infinity = unsafe { crate::ffi::SCIPinfinity(model.scip.raw) };

                // c1 is `2 x1 + x2 <= 100`.
                assert_eq!(row.n_non_zeroes(), 2);
                assert!(row.lhs() <= -infinity);
                assert_eq!(row.rhs(), 100.0);
                assert_eq!(row.index(), 0);
                assert!(!row.is_modifiable());
                assert!(!row.is_removable());
                assert!(!row.is_local());
                assert!(!row.is_integral());
                assert!(row.constraint().is_some());
                // The row is binding at its rhs in the optimal LP, so its slack is
                // nonbasic at the upper bound.
                assert_eq!(row.basis_status(), crate::BasisStatus::Upper);
                assert_eq!(row.origin_type(), crate::RowOrigin::Constraint);
                assert!(!row.is_in_global_cut_pool());
                assert!(row.is_in_lp());
                assert_eq!(row.lp_position(), Some(0));
                assert_eq!(row.depth(), 0);
                assert_eq!(row.active_lp_count(), 1);
                assert_eq!(row.n_lp_since_create(), 1);
                assert_eq!(row.rank(), 0);
                row.set_rank(1);
                assert_eq!(row.rank(), 1);
                assert_eq!(row.name(), "c1");
                assert_eq!(row.age(), 0);
                // Dual value of c1 is 2/3, negated for the internally-minimized sense.
                assert!((row.dual() - (-2.0 / 3.0)).abs() < 1e-9);
                // The LP is feasible, so there is no Farkas (infeasibility) certificate.
                assert!(row.farkas_dual() >= infinity);
                let cols = row.cols();
                assert_eq!(cols.len(), 2);
                assert_eq!(cols[0].index(), 0);

                self.checked.store(true, Ordering::SeqCst);
            }
        }

        let checked = Rc::new(AtomicBool::new(false));
        let mut model = row_test_model();
        model.add(
            eventhdlr(RowTesterEventHandler {
                checked: checked.clone(),
            })
            .name("RowTesterEventHandler"),
        );
        model.solve();
        assert!(checked.load(Ordering::SeqCst), "row assertions never ran");
    }

    /// Same LP as the `col` test: maximize `3 x1 + 4 x2` s.t. `2 x1 + x2 <= 100`
    /// and `x1 + 2 x2 <= 80`, both continuous. `c1` is the first constraint.
    fn row_test_model() -> Model<crate::ProblemCreated> {
        let mut model = Model::new()
            .hide_output()
            .include_default_plugins()
            .create_prob("test")
            .set_obj_sense(ObjSense::Maximize)
            .set_presolving(ParamSetting::Off)
            .set_separating(ParamSetting::Off)
            .set_heuristics(ParamSetting::Off);

        let x1 = model.add_var(0.0, f64::INFINITY, 3.0, "x1", VarType::Continuous);
        let x2 = model.add_var(0.0, f64::INFINITY, 4.0, "x2", VarType::Continuous);
        model.add_cons(vec![&x1, &x2], &[2.0, 1.0], f64::NEG_INFINITY, 100.0, "c1");
        model.add_cons(vec![&x1, &x2], &[1.0, 2.0], f64::NEG_INFINITY, 80.0, "c2");
        model
    }
}
