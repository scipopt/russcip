use crate::ffi;

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

impl From<PricerResultState> for u32 {
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
        model::{Model, ModelRef, ProblemCreated},
        status::Status,
        variable::VarType,
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
        let mut pricer = PanickingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, &mut pricer);

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
        let mut pricer = LyingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, &mut pricer);

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
        let mut pricer = EarlyStoppingPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, &mut pricer);

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
        let mut pricer = OptimalPricer {};

        let model = crate::model::Model::new()
            .hide_output()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .include_pricer("", "", 9999999, false, &mut pricer);

        let solved = model.solve();
        assert_eq!(solved.get_status(), Status::Optimal);
    }

    struct AddSameColumnPricer {
        added: bool,
        model: ModelRef<Model<ProblemCreated>>,
    }

    impl Pricer for AddSameColumnPricer {
        fn generate_columns(&mut self, _farkas: bool) -> PricerResult {
            if self.added {
                PricerResult {
                    state: PricerResultState::NoColumns,
                    lower_bound: None,
                }
            } else {
                self.added = true;
                self.model
                    .add_priced_var(0.0, 1.0, 1.0, "x", VarType::Binary);
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

        let mut pricer = AddSameColumnPricer {
            added: false,
            model: ModelRef::new(&mut model),
        };

        let solved = model
            .include_pricer("", "", 9999999, false, &mut pricer)
            .solve();
        assert_eq!(solved.get_status(), Status::Optimal);
    }
}
