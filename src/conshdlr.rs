use crate::{ffi, scip_call, Model, Retcode, Row, Solution, Solving};
use scip_sys::SCIP_ROW;

/// A trait for implementing custom constraint handlers.
pub trait Conshdlr {
    /// Check if the (primal) solution satisfies the constraint.
    fn check(&mut self, model: Model<Solving>, conshdlr: SCIPConshdlr, solution: &Solution)
        -> bool;

    /// Enforce the constraint for the current sub-problem's (LP) solution.
    fn enforce(&mut self, model: Model<Solving>, conshdlr: SCIPConshdlr) -> ConshdlrResult;
}

/// The result of enforcing a constraint handler.
pub enum ConshdlrResult {
    /// States that the problem is feasible.
    Feasible,
    /// States that the problem is infeasible.
    CutOff,
    /// Added another constraint that resolves the infeasibility.
    ConsAdded,
    /// Reduced the domain of a variable.
    ReducedDom,
    /// Added a cutting plane that separates the lp solution.
    Separated,
    /// Request to resolve the LP.
    SolveLP,
    /// Created a branching.
    Branched,
}

impl From<ConshdlrResult> for ffi::SCIP_Result {
    fn from(result: ConshdlrResult) -> Self {
        match result {
            ConshdlrResult::Feasible => ffi::SCIP_Result_SCIP_FEASIBLE,
            ConshdlrResult::CutOff => ffi::SCIP_Result_SCIP_CUTOFF,
            ConshdlrResult::ConsAdded => ffi::SCIP_Result_SCIP_CONSADDED,
            ConshdlrResult::ReducedDom => ffi::SCIP_Result_SCIP_REDUCEDDOM,
            ConshdlrResult::Separated => ffi::SCIP_Result_SCIP_SEPARATED,
            ConshdlrResult::SolveLP => ffi::SCIP_Result_SCIP_SOLVELP,
            ConshdlrResult::Branched => ffi::SCIP_Result_SCIP_BRANCHED,
        }
    }
}

/// Wrapper for the internal SCIP constraint handler.
pub struct SCIPConshdlr {
    pub(crate) raw: *mut ffi::SCIP_CONSHDLR,
}

impl SCIPConshdlr {
    /// Returns a raw pointer to the underlying `ffi::SCIP_CONSHDLR` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_CONSHDLR {
        self.raw
    }

    /// Returns the name of the constraint handler.
    pub fn name(&self) -> String {
        let name = unsafe { ffi::SCIPconshdlrGetName(self.raw) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    /// Returns the description of the constraint handler.
    pub fn desc(&self) -> String {
        let desc = unsafe { ffi::SCIPconshdlrGetDesc(self.raw) };
        let desc = unsafe { std::ffi::CStr::from_ptr(desc) };
        desc.to_str().unwrap().to_string()
    }

    /// Creates an empty row for the constraint handler.
    pub fn create_empty_row(
        &self,
        model: &Model<Solving>,
        name: &str,
        lhs: f64,
        rhs: f64,
        local: bool,
        modifiable: bool,
        removable: bool,
    ) -> Result<Row, Retcode> {
        let name = std::ffi::CString::new(name).unwrap();
        let local = if local { 1 } else { 0 };
        let modifiable = if modifiable { 1 } else { 0 };
        let removable = if removable { 1 } else { 0 };

        let mut row: *mut SCIP_ROW = std::ptr::null_mut();
        scip_call! { ffi::SCIPcreateEmptyRowConshdlr(model.scip.raw, &mut row, self.raw, name.as_ptr(), lhs, rhs, local, modifiable, removable) }

        Ok(Row {
            raw: row,
            scip: model.scip.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Status;

    #[test]
    fn all_inf_conshdlr() {
        struct AllInfeasibleConshdlr;

        impl Conshdlr for AllInfeasibleConshdlr {
            fn check(
                &mut self,
                _model: Model<Solving>,
                _conshdlr: SCIPConshdlr,
                _solution: &Solution,
            ) -> bool {
                false
            }

            fn enforce(
                &mut self,
                _model: Model<Solving>,
                _conshdlr: SCIPConshdlr,
            ) -> ConshdlrResult {
                ConshdlrResult::CutOff
            }
        }

        let mut model = Model::default();

        model.include_conshdlr(
            "AllInfeasibleConshdlr",
            "All infeasible constraint handler",
            -1,
            -1,
            Box::new(AllInfeasibleConshdlr {}),
        );

        let solved = model.solve();

        assert_eq!(solved.status(), Status::Infeasible);
    }
}
