use crate::{Model, Retcode, Row, Solving, ffi, scip_call};
use scip_sys::{SCIP_ROW, SCIP_Result};
use std::fmt::Debug;

/// A trait for defining custom separation routines.
pub trait Separator {
    /// Execute the separation routine on LP solutions.
    ///
    /// # Arguments
    /// * `model` - the current model of the SCIP instance in `Solving` stage.
    /// * `sep` - the internal separator object.
    ///
    /// # Returns
    /// * `SeparationResult` indicating the result of the separation routine.
    fn execute_lp(&mut self, model: Model<Solving>, sep: SCIPSeparator) -> SeparationResult;
}

/// The result of a separation routine.
pub enum SeparationResult {
    /// Detected that the node is infeasible in the variable's bounds and can be cut off
    Cutoff,
    /// Added a constraint to the problem
    ConsAdded,
    /// Reduced the domain of a variable
    ReducedDomain,
    /// Added a cutting plane to the LP
    Separated,
    /// The separator searched, but did not find domain reductions, cutting planes, or cut constraints
    DidNotFind,
    /// The separator was skipped
    DidNotRun,
    /// The separator was skipped, but should be called again
    Delayed,
    /// A new separation round should be started without calling the remaining separator methods
    NewRound,
}

impl From<SCIP_Result> for SeparationResult {
    fn from(result: SCIP_Result) -> Self {
        match result {
            ffi::SCIP_Result_SCIP_CUTOFF => SeparationResult::Cutoff,
            ffi::SCIP_Result_SCIP_CONSADDED => SeparationResult::ConsAdded,
            ffi::SCIP_Result_SCIP_REDUCEDDOM => SeparationResult::ReducedDomain,
            ffi::SCIP_Result_SCIP_SEPARATED => SeparationResult::Separated,
            ffi::SCIP_Result_SCIP_DIDNOTFIND => SeparationResult::DidNotFind,
            ffi::SCIP_Result_SCIP_DIDNOTRUN => SeparationResult::DidNotRun,
            ffi::SCIP_Result_SCIP_DELAYED => SeparationResult::Delayed,
            ffi::SCIP_Result_SCIP_NEWROUND => SeparationResult::NewRound,
            _ => panic!("Unknown SCIP result"),
        }
    }
}

impl From<SeparationResult> for SCIP_Result {
    fn from(val: SeparationResult) -> Self {
        match val {
            SeparationResult::Cutoff => ffi::SCIP_Result_SCIP_CUTOFF,
            SeparationResult::ConsAdded => ffi::SCIP_Result_SCIP_CONSADDED,
            SeparationResult::ReducedDomain => ffi::SCIP_Result_SCIP_REDUCEDDOM,
            SeparationResult::Separated => ffi::SCIP_Result_SCIP_SEPARATED,
            SeparationResult::DidNotFind => ffi::SCIP_Result_SCIP_DIDNOTFIND,
            SeparationResult::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            SeparationResult::Delayed => ffi::SCIP_Result_SCIP_DELAYED,
            SeparationResult::NewRound => ffi::SCIP_Result_SCIP_NEWROUND,
        }
    }
}

/// A wrapper struct for the internal ffi::SCIP_SEPA
#[derive(Debug)]
pub struct SCIPSeparator {
    pub(crate) raw: *mut ffi::SCIP_SEPA,
}

impl SCIPSeparator {
    /// Returns the internal raw pointer of the separator.
    pub fn inner(&self) -> *mut ffi::SCIP_SEPA {
        self.raw
    }

    /// Returns the name of the separator.
    pub fn name(&self) -> String {
        unsafe {
            let name_ptr = ffi::SCIPsepaGetName(self.raw);
            let name = std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap();
            name.to_string()
        }
    }

    /// Returns the description of the separator.
    pub fn desc(&self) -> String {
        unsafe {
            let desc_ptr = ffi::SCIPsepaGetDesc(self.raw);
            let desc = std::ffi::CStr::from_ptr(desc_ptr).to_str().unwrap();
            desc.to_string()
        }
    }

    /// Returns the priority of the separator.
    pub fn priority(&self) -> i32 {
        unsafe { ffi::SCIPsepaGetPriority(self.raw) }
    }

    /// Returns the frequency of the separator.
    pub fn freq(&self) -> i32 {
        unsafe { ffi::SCIPsepaGetFreq(self.raw) }
    }

    /// Set the frequency of the separator.
    pub fn set_freq(&mut self, freq: i32) {
        unsafe { ffi::SCIPsepaSetFreq(self.raw, freq) }
    }

    /// Returns the maxbounddist of the separator.
    pub fn maxbounddist(&self) -> f64 {
        unsafe { ffi::SCIPsepaGetMaxbounddist(self.raw) }
    }

    /// Returns whether the separator is delayed.
    pub fn is_delayed(&self) -> bool {
        (unsafe { ffi::SCIPsepaIsDelayed(self.raw) }) != 0
    }

    /// Creates an empty LP row.
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
        scip_call! { ffi::SCIPcreateEmptyRowSepa(model.scip.raw, &mut row, self.raw, name.as_ptr(), lhs, rhs, local, modifiable, removable) }

        Ok(Row {
            raw: row,
            scip: model.scip.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::row::RowSource;
    use crate::prelude::{cons, row, sepa, var};
    use crate::{
        Model, ModelWithProblem, ObjSense, ProblemOrSolving, RowOrigin, Solving, VarType, Variable,
        minimal_model,
    };

    struct NotRunningSeparator;

    impl Separator for NotRunningSeparator {
        fn execute_lp(&mut self, _model: Model<Solving>, _sepa: SCIPSeparator) -> SeparationResult {
            SeparationResult::DidNotRun
        }
    }

    #[test]
    fn test_not_running_separator() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let sep = NotRunningSeparator;

        model.add(
            sepa(sep)
                .name("NotRunningSeparator")
                .desc("Does not run the separation routine"),
        );

        model.solve();
    }

    struct ConsAddingSeparator {}

    impl Separator for ConsAddingSeparator {
        fn execute_lp(
            &mut self,
            mut model: Model<Solving>,
            _sepa: SCIPSeparator,
        ) -> SeparationResult {
            let vars = model.vars();
            let var_refs: Vec<&Variable> = vars.iter().collect();
            let varlen = vars.len();

            model.add(cons().eq(5.0).coefs(var_refs, vec![1.0; varlen]));
            SeparationResult::ConsAdded
        }
    }

    #[test]
    fn cons_adding_separator() {
        let mut model = minimal_model()
            .hide_output()
            .set_obj_sense(ObjSense::Maximize);

        let x = model.add(var().bin().obj(1.0));
        let y = model.add(var().bin().obj(1.0));

        model.add(cons().eq(1.0).coef(&x, 1.0).coef(&y, 1.0));

        let sep = ConsAddingSeparator {};

        model.add(
            sepa(sep)
                .name("ConsAddingSeparator")
                .desc("Adds a constraint to the model"),
        );
        let solved = model.solve();

        assert_eq!(solved.status(), crate::Status::Infeasible);
    }

    struct InternalSeparatorDataTester;

    impl Separator for InternalSeparatorDataTester {
        fn execute_lp(
            &mut self,
            mut model: Model<Solving>,
            sep: SCIPSeparator,
        ) -> SeparationResult {
            assert_eq!(sep.name(), "InternalSeparatorDataTester");
            assert_eq!(sep.desc(), "Internal separator data tester");
            assert_eq!(sep.priority(), 1000000);
            assert_eq!(sep.freq(), 1);
            assert_eq!(sep.maxbounddist(), 1.0);
            assert!(!sep.is_delayed());

            let row = model.add(
                row()
                    .bounds(0.0, 1.0)
                    .removable(false)
                    .local(false)
                    .modifiable(true)
                    .name("test")
                    .source(RowSource::Separator(&sep)),
            );
            assert_eq!(row.name(), "test");
            assert_eq!(row.lhs(), 0.0);
            assert_eq!(row.rhs(), 1.0);
            assert_eq!(row.n_non_zeroes(), 0);
            assert!(row.is_modifiable());
            assert!(!row.is_local());
            assert!(!row.is_removable());
            assert_eq!(row.origin_type(), RowOrigin::Separator);

            SeparationResult::DidNotRun
        }
    }

    #[test]
    fn test_internal_scip_separator() {
        let mut model = Model::new()
            .hide_output()
            .set_longint_param("limits/nodes", 2)
            .unwrap() // only call brancher once
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap();

        let sep = InternalSeparatorDataTester;

        model.add(
            sepa(sep)
                .name("InternalSeparatorDataTester")
                .desc("Internal separator data tester")
                .priority(1000000)
                .freq(1)
                .maxbounddist(1.0)
                .usesubscip(false)
                .delay(false),
        );

        model.solve();
    }

    struct CutsAddingSeparator;

    impl Separator for CutsAddingSeparator {
        fn execute_lp(
            &mut self,
            mut model: Model<Solving>,
            sepa: SCIPSeparator,
        ) -> SeparationResult {
            let mut row = model.add(
                row()
                    .name("test")
                    .eq(5.0)
                    .local(true)
                    .modifiable(false)
                    .removable(false)
                    .source(RowSource::Separator(&sepa)),
            );
            assert_eq!(row.name(), "test");
            assert_eq!(row.lhs(), 5.0);
            assert_eq!(row.rhs(), 5.0);
            assert_eq!(row.n_non_zeroes(), 0);
            assert!(row.is_local());
            assert!(!row.is_modifiable());
            assert!(!row.is_removable());
            assert_eq!(row.origin_type(), RowOrigin::Separator);

            let vars = model.vars();
            for var in vars.clone() {
                row.set_coeff(&var, 1.0);
            }
            model.add_cut(row, true);
            let n_conss_before = model.n_conss();
            model.add_cons_local(&cons().ge(7.0).coef(&(vars[0]), 2.).coef(&(vars[1]), 1.));
            assert_eq!(model.n_conss(), n_conss_before + 1);

            SeparationResult::Separated
        }
    }

    #[test]
    fn cuts_adding() {
        let mut model = minimal_model()
            .hide_output()
            .set_obj_sense(ObjSense::Maximize);

        let x = model.add_var(0.0, 1.0, 1.0, "x", VarType::Binary);
        let y = model.add_var(0.0, 1.0, 1.0, "y", VarType::Binary);

        model.add_cons(vec![&x, &y], &[1.0, 1.0], 1.0, 1.0, "cons1");

        let sep = CutsAddingSeparator {};
        model.add(
            sepa(sep)
                .name("CutsAddingSeparator")
                .desc("Adds a cut to the model"),
        );
        let solved = model.solve();

        assert_eq!(solved.status(), crate::Status::Infeasible);
    }
}
