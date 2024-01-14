use crate::ffi;
use scip_sys::SCIP_Status;

/// An enum representing the status of a SCIP optimization run.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    /// The solving status is not yet known.
    Unknown,
    /// The user interrupted the solving process (by pressing CTRL-C).
    UserInterrupt,
    /// The solving process was interrupted because the node limit was reached.
    NodeLimit,
    /// The solving process was interrupted because the total node limit was reached (incl. restarts).
    TotalNodeLimit,
    /// The solving process was interrupted because the stalling node limit was reached (no improvement w.r.t. primal bound).
    StallNodeLimit,
    /// The solving process was interrupted because the time limit was reached.
    TimeLimit,
    /// The solving process was interrupted because the memory limit was reached.
    MemoryLimit,
    /// The solving process was interrupted because the gap limit was reached.
    GapLimit,
    /// The solving process was interrupted because the solution limit was reached.
    SolutionLimit,
    /// The solving process was interrupted because the solution improvement limit was reached.
    BestSolutionLimit,
    /// The solving process was interrupted because the restart limit was reached.
    RestartLimit,
    /// The problem was solved to optimality, an optimal solution is available.
    Optimal,
    /// The problem was proven to be infeasible.
    Infeasible,
    /// The problem was proven to be unbounded.
    Unbounded,
    /// The problem was proven to be either infeasible or unbounded.
    Inforunbd,
    /// Status if the process received a SIGTERM signal.
    Terminate,
}

impl From<SCIP_Status> for Status {
    /// Converts a u32 value to a `Status` enum variant.
    fn from(val: SCIP_Status) -> Self {
        match val {
            ffi::SCIP_Status_SCIP_STATUS_UNKNOWN => Status::Unknown,
            ffi::SCIP_Status_SCIP_STATUS_USERINTERRUPT => Status::UserInterrupt,
            ffi::SCIP_Status_SCIP_STATUS_NODELIMIT => Status::NodeLimit,
            ffi::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT => Status::TotalNodeLimit,
            ffi::SCIP_Status_SCIP_STATUS_STALLNODELIMIT => Status::StallNodeLimit,
            ffi::SCIP_Status_SCIP_STATUS_TIMELIMIT => Status::TimeLimit,
            ffi::SCIP_Status_SCIP_STATUS_MEMLIMIT => Status::MemoryLimit,
            ffi::SCIP_Status_SCIP_STATUS_GAPLIMIT => Status::GapLimit,
            ffi::SCIP_Status_SCIP_STATUS_SOLLIMIT => Status::SolutionLimit,
            ffi::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT => Status::BestSolutionLimit,
            ffi::SCIP_Status_SCIP_STATUS_RESTARTLIMIT => Status::RestartLimit,
            ffi::SCIP_Status_SCIP_STATUS_OPTIMAL => Status::Optimal,
            ffi::SCIP_Status_SCIP_STATUS_INFEASIBLE => Status::Infeasible,
            ffi::SCIP_Status_SCIP_STATUS_UNBOUNDED => Status::Unbounded,
            ffi::SCIP_Status_SCIP_STATUS_INFORUNBD => Status::Inforunbd,
            ffi::SCIP_Status_SCIP_STATUS_TERMINATE => Status::Terminate,
            _ => panic!("Unknown SCIP status {val:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Model;

    #[test]
    fn time_limit() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/time", 0.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::TimeLimit);
    }

    #[test]
    fn memory_limit() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/memory", 0.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::MemoryLimit);
    }

    #[test]
    fn gap_limit() {
        let model = Model::new()
            .hide_output()
            .set_real_param("limits/gap", 100000.)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/gen-ip054.mps")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::GapLimit);
    }

    #[test]
    fn solution_limit() {
        let model = Model::new()
            .hide_output()
            .set_int_param("limits/solutions", 0)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::SolutionLimit);
    }

    #[test]
    fn total_node_limit() {
        let model = Model::new()
            .hide_output()
            .set_longint_param("limits/totalnodes", 0)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::TotalNodeLimit);
    }

    #[test]
    fn stall_node_limit() {
        let model = Model::new()
            .hide_output()
            .set_longint_param("limits/stallnodes", 0)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::StallNodeLimit);
    }

    #[test]
    fn best_solution_limit() {
        let model = Model::new()
            .hide_output()
            .set_int_param("limits/bestsol", 0)
            .unwrap()
            .include_default_plugins()
            .read_prob("data/test/simple.lp")
            .unwrap()
            .solve();

        assert_eq!(model.status(), Status::BestSolutionLimit);
    }

    #[test]
    fn unknown() {
        assert_eq!(Model::new().status(), Status::Unknown);
    }
}
