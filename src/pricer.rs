use crate::ffi;

pub trait Pricer {
    /// Generates negative reduced cost columns.
    ///
    /// # Arguments
    /// farkas: bool
    /// If true, the pricer should generate columns to repair feasibility of LP.
    fn generate_columns(&mut self, farkas: bool) -> PricerResult;
}

#[derive(Debug, PartialEq)]
pub enum PricerResultState {
    /// Pricer did not run
    DidNotRun,
    /// Pricer added new columns with negative reduced cost
    FoundColumns,
    /// Pricer did not find any columns with negative reduced cost (i.e. current LP solution is optimal)
    NoColumns,
    /// Pricer wants to perform early branching
    StopEarly,
}

pub struct PricerResult {
    pub state: PricerResultState,
    pub lower_bound: Option<f64>,
}

impl From<PricerResultState> for u32 {
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
    use crate::status::Status;

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
}
