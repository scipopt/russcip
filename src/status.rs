use crate::c_api;

#[derive(Debug)]
pub enum SCIPStatus {
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

impl SCIPStatus {
    pub fn from_c_scip_status(val: c_api::SCIP_Status) -> Option<Self> {
        match val {
            c_api::SCIP_Status_SCIP_STATUS_UNKNOWN => Some(SCIPStatus::UNKNOWN),
            c_api::SCIP_Status_SCIP_STATUS_USERINTERRUPT => Some(SCIPStatus::USERINTERRUPT),
            c_api::SCIP_Status_SCIP_STATUS_NODELIMIT => Some(SCIPStatus::NODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT => Some(SCIPStatus::TOTALNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_STALLNODELIMIT => Some(SCIPStatus::STALLNODELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_TIMELIMIT => Some(SCIPStatus::TIMELIMIT),
            c_api::SCIP_Status_SCIP_STATUS_MEMLIMIT => Some(SCIPStatus::MEMLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_GAPLIMIT => Some(SCIPStatus::GAPLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_SOLLIMIT => Some(SCIPStatus::SOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT => Some(SCIPStatus::BESTSOLLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_RESTARTLIMIT => Some(SCIPStatus::RESTARTLIMIT),
            c_api::SCIP_Status_SCIP_STATUS_OPTIMAL => Some(SCIPStatus::OPTIMAL),
            c_api::SCIP_Status_SCIP_STATUS_INFEASIBLE => Some(SCIPStatus::INFEASIBLE),
            c_api::SCIP_Status_SCIP_STATUS_UNBOUNDED => Some(SCIPStatus::UNBOUNDED),
            c_api::SCIP_Status_SCIP_STATUS_INFORUNBD => Some(SCIPStatus::INFORUNBD),
            c_api::SCIP_Status_SCIP_STATUS_TERMINATE => Some(SCIPStatus::TERMINATE),
            _ => None,
        }
    }
}