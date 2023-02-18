use crate::ffi;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    UNKNOWN,
    USERINTERRUPT,
    NODELIMIT,
    TOTALNODELIMIT,
    STALLNODELIMIT,
    TIMELIMIT,
    MEMLIMIT,
    GAPLIMIT,
    SOLLIMIT,
    BESTSOLLIMIT,
    RESTARTLIMIT,
    OPTIMAL,
    INFEASIBLE,
    UNBOUNDED,
    INFORUNBD,
    TERMINATE,
}

impl Status {
    pub fn from_c_scip_status(val: ffi::SCIP_Status) -> Option<Self> {
        match val {
            ffi::SCIP_Status_SCIP_STATUS_UNKNOWN => Some(Status::UNKNOWN),
            ffi::SCIP_Status_SCIP_STATUS_USERINTERRUPT => Some(Status::USERINTERRUPT),
            ffi::SCIP_Status_SCIP_STATUS_NODELIMIT => Some(Status::NODELIMIT),
            ffi::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT => Some(Status::TOTALNODELIMIT),
            ffi::SCIP_Status_SCIP_STATUS_STALLNODELIMIT => Some(Status::STALLNODELIMIT),
            ffi::SCIP_Status_SCIP_STATUS_TIMELIMIT => Some(Status::TIMELIMIT),
            ffi::SCIP_Status_SCIP_STATUS_MEMLIMIT => Some(Status::MEMLIMIT),
            ffi::SCIP_Status_SCIP_STATUS_GAPLIMIT => Some(Status::GAPLIMIT),
            ffi::SCIP_Status_SCIP_STATUS_SOLLIMIT => Some(Status::SOLLIMIT),
            ffi::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT => Some(Status::BESTSOLLIMIT),
            ffi::SCIP_Status_SCIP_STATUS_RESTARTLIMIT => Some(Status::RESTARTLIMIT),
            ffi::SCIP_Status_SCIP_STATUS_OPTIMAL => Some(Status::OPTIMAL),
            ffi::SCIP_Status_SCIP_STATUS_INFEASIBLE => Some(Status::INFEASIBLE),
            ffi::SCIP_Status_SCIP_STATUS_UNBOUNDED => Some(Status::UNBOUNDED),
            ffi::SCIP_Status_SCIP_STATUS_INFORUNBD => Some(Status::INFORUNBD),
            ffi::SCIP_Status_SCIP_STATUS_TERMINATE => Some(Status::TERMINATE),
            _ => None,
        }
    }
}
