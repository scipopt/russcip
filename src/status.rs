use crate::c_api;

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
    pub fn from_c_scip_status(val: c_api::SCIP_Status) -> Option<Self> {
        match val {
            c_api::SCIP_Status_SCIP_STATUS_UNKNOWN => Some(Status::UNKNOWN),
            c_api::SCIP_Status_SCIP_STATUS_USERINTERRUPT => Some(Status::USERINTERRUPT),
            c_api::SCIP_Status_SCIP_STATUS_NODELIMIT => Some(Status::NODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT => Some(Status::TOTALNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_STALLNODELIMIT => Some(Status::STALLNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TIMELIMIT => Some(Status::TIMELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_MEMLIMIT => Some(Status::MEMLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_GAPLIMIT => Some(Status::GAPLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_SOLLIMIT => Some(Status::SOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT => Some(Status::BESTSOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_RESTARTLIMIT => Some(Status::RESTARTLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_OPTIMAL => Some(Status::OPTIMAL),
            c_api::SCIP_Status_SCIP_STATUS_INFEASIBLE => Some(Status::INFEASIBLE),
            c_api::SCIP_Status_SCIP_STATUS_UNBOUNDED => Some(Status::UNBOUNDED),
            c_api::SCIP_Status_SCIP_STATUS_INFORUNBD => Some(Status::INFORUNBD),
            c_api::SCIP_Status_SCIP_STATUS_TERMINATE => Some(Status::TERMINATE),
            _ => None,
        }
    }
}