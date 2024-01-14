use crate::ffi;
use scip_sys::SCIP_Result;

/// A trait for SCIP pricers.
pub trait Pricer {
    /// Generates negative reduced cost columns.
    ///
    /// # Arguments
    /// * `farkas`: If true, the pricer should generate columns to repair feasibility of LP.
    fn generate_columns(&mut self, farkas: bool) -> PricerResult;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{Model, ModelWithProblem},
        status::Status,
        variable::VarType,
        ProblemOrSolving, Solving,
    };

    struct PanickingPricer;

    impl Pricer for PanickingPricer {
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
            panic!("Not implemented")
        }
    }

    #[test]
    #[should_panic]
    fn panicking_pricer() {
        let pricer = PanickingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, Box::new(pricer));

        // solve model
        model.solve();
    }

    struct LyingPricer;

    impl Pricer for LyingPricer {
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
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
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
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
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
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
        model: Model<Solving>,
        data: ComplexData,
    }

    impl Pricer for AddSameColumnPricer {
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
            assert!(self.data.a == (0..1000).collect::<Vec<usize>>());
            if self.added {
                PricerResult {
                    state: PricerResultState::NoColumns,
                    lower_bound: None,
                }
            } else {
                self.added = true;
                let nvars_before = self.model.n_vars();
                let var = self
                    .model
                    .add_priced_var(0.0, 1.0, 1.0, "x", VarType::Binary);
                let conss = self.model.conss();
                for cons in conss {
                    self.model.add_cons_coef(cons, var.clone(), 1.0);
                }
                let nvars_after = self.model.n_vars();
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
            model: model.clone_for_plugins(),
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
}
