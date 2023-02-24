use crate::ffi;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Unknown,
    UserInterrupt,
    NodeLimit,
    TotalNodeLimit,
    StallNodeLimit,
    TimeLimit,
    MemoryLimit,
    GapLimit,
    SolutionLimit,
    BestSolutionLimit,
    RestartLimit,
    Optimal,
    Infeasible,
    Unbounded,
    Inforunbd,
    Terminate,
}

impl From<u32> for Status {
    fn from(val: u32) -> Self {
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
            _ => panic!("Unknown SCIP status {:?}", val),
        }
    }
}
