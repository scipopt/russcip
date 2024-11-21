use crate::ffi;
use scip_sys::SCIP_Result;

/// A trait for defining custom separation routines.
pub trait Separator {
    /// Execute the separation routine on LP solutions.
    fn execute_lp(&mut self) -> SeparationResult;
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

impl Into<SCIP_Result> for SeparationResult {
    fn into(self) -> SCIP_Result {
        match self {
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