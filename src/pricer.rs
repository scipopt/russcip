use crate::{ffi, Model, Solving};
use scip_sys::SCIP_Result;

/// A trait for SCIP pricers.
pub trait Pricer {
    /// Generates negative reduced cost columns.
    ///
    /// # Arguments
    /// * `model`: the current model of the SCIP instance in `Solving` stage.
    /// * `pricer`: the internal pricer object.
    /// * `farkas`: If true, the pricer should generate columns to repair feasibility of LP.
    fn generate_columns(
        &mut self,
        model: Model<Solving>,
        pricer: SCIPPricer,
        farkas: bool,
    ) -> PricerResult;
}

/// An enum representing the possible states of a `PricerResult`.
#[derive(Debug, PartialEq)]
pub enum PricerResultState {
    /// The pricer did not run.
    DidNotRun,
    /// The pricer added new columns with negative reduced cost.
    FoundColumns,
    /// The pricer did not find any columns with negative reduced cost (i.e. current LP solution is optimal).
    NoColumns,
    /// The pricer wants to perform early branching.
    StopEarly,
}

/// A struct representing the result of a pricer.
pub struct PricerResult {
    /// The state of the pricer result.
    pub state: PricerResultState,
    /// A calculated lower bound on the objective value of the current node.
    pub lower_bound: Option<f64>,
}

impl From<PricerResultState> for SCIP_Result {
    /// Converts a `PricerResultState` enum variant to an `SCIP_Result` value.
    fn from(val: PricerResultState) -> Self {
        match val {
            PricerResultState::DidNotRun => ffi::SCIP_Result_SCIP_DIDNOTRUN,
            PricerResultState::FoundColumns
            | PricerResultState::StopEarly
            | PricerResultState::NoColumns => ffi::SCIP_Result_SCIP_SUCCESS,
        }
    }
}

/// A wrapper around a SCIP pricer object.
pub struct SCIPPricer {
    pub(crate) raw: *mut ffi::SCIP_PRICER,
}

impl SCIPPricer {
    /// Returns the internal raw pointer of the pricer.
    pub fn inner(&self) -> *mut ffi::SCIP_PRICER {
        self.raw
    }

    /// Returns the name of the pricer.
    pub fn name(&self) -> String {
        unsafe {
            let name = ffi::SCIPpricerGetName(self.raw);
            std::ffi::CStr::from_ptr(name)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the description of the pricer.
    pub fn desc(&self) -> String {
        unsafe {
            let desc = ffi::SCIPpricerGetDesc(self.raw);
            std::ffi::CStr::from_ptr(desc)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the priority of the pricer.
    pub fn priority(&self) -> i32 {
        unsafe { ffi::SCIPpricerGetPriority(self.raw) }
    }

    /// Returns the delay of the pricer.
    pub fn is_delayed(&self) -> bool {
        unsafe { ffi::SCIPpricerIsDelayed(self.raw) != 0 }
    }

    /// Returns whether the pricer is active.
    pub fn is_active(&self) -> bool {
        unsafe { ffi::SCIPpricerIsActive(self.raw) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::ModelWithProblem, status::Status, variable::VarType, Model, ProblemOrSolving,
        Solving,
    };

    struct LyingPricer;

    impl Pricer for LyingPricer {
        fn generate_columns(
            &mut self,
            _model: Model<Solving>,
            _pricer: SCIPPricer,
            _farkas: bool,
        ) -> PricerResult {
            PricerResult {
                state: PricerResultState::FoundColumns,
                lower_bound: None,
            }
        }
    }

    #[test]
    #[should_panic]
    fn nothing_pricer() {
        let pricer = LyingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, Box::new(pricer));

        model.solve();
    }

    struct EarlyStoppingPricer;

    impl Pricer for EarlyStoppingPricer {
        fn generate_columns(
            &mut self,
            _model: Model<Solving>,
            _pricer: SCIPPricer,
            _farkas: bool,
        ) -> PricerResult {
            PricerResult {
                state: PricerResultState::StopEarly,
                lower_bound: None,
            }
        }
    }

    #[test]
    #[should_panic]
    /// Stops pricing early then throws an error that no branching can be performed
    fn early_stopping_pricer() {
        let pricer = EarlyStoppingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, Box::new(pricer));

        model.solve();
    }

    struct OptimalPricer;

    impl Pricer for OptimalPricer {
        fn generate_columns(
            &mut self,
            _model: Model<Solving>,
            _pricer: SCIPPricer,
            _farkas: bool,
        ) -> PricerResult {
            PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            }
        }
    }

    #[test]
    fn optimal_pricer() {
        let pricer = OptimalPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, Box::new(pricer));

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
    }

    #[derive(Debug, PartialEq, Clone)]
    struct ComplexData {
        a: Vec<usize>,
        b: f64,
        c: Option<isize>,
    }

    struct AddSameColumnPricer {
        added: bool,
        data: ComplexData,
    }

    impl Pricer for AddSameColumnPricer {
        fn generate_columns(
            &mut self,
            mut model: Model<Solving>,
            _pricer: SCIPPricer,
            _farkas: bool,
        ) -> PricerResult {
            assert_eq!(self.data.a, (0..1000).collect::<Vec<usize>>());
            if self.added {
                PricerResult {
                    state: PricerResultState::NoColumns,
                    lower_bound: None,
                }
            } else {
                self.added = true;
                let nvars_before = model.n_vars();
                let var = model.add_priced_var(0.0, 1.0, 1.0, "x", VarType::Binary);
                let conss = model.conss();
                for cons in conss {
                    model.add_cons_coef(cons, &var, 1.0);
                }
                let nvars_after = model.n_vars();
                assert_eq!(nvars_before + 1, nvars_after);
                PricerResult {
                    state: PricerResultState::FoundColumns,
                    lower_bound: None,
                }
            }
        }
    }

    #[test]
    fn add_same_column_pricer() {
        let mut model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap();

        let conss = model.conss();
        for c in conss {
            model.set_cons_modifiable(c, true);
        }

        let pricer = AddSameColumnPricer {
            added: false,
            data: ComplexData {
                a: (0..1000).collect::<Vec<usize>>(),
                b: 1.0,
                c: Some(1),
            },
        };

        let solved = model
            .include_pricer("", "", 9999999, false, Box::new(pricer))
            .solve();
        assert_eq!(solved.status(), Status::Optimal);
    }

    struct InternalSCIPPricerTester;

    impl Pricer for InternalSCIPPricerTester {
        fn generate_columns(
            &mut self,
            _model: Model<Solving>,
            pricer: SCIPPricer,
            _farkas: bool,
        ) -> PricerResult {
            assert_eq!(pricer.name(), "internal");
            assert_eq!(pricer.desc(), "internal pricer");
            assert_eq!(pricer.priority(), 100);
            assert!(!pricer.is_delayed());
            assert!(pricer.is_active());
            PricerResult {
                state: PricerResultState::NoColumns,
                lower_bound: None,
            }
        }
    }

    #[test]
    fn internal_pricer() {
        let pricer = InternalSCIPPricerTester {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("internal", "internal pricer", 100, false, Box::new(pricer));

        model.solve();
    }
}
