use crate::c_api;

#[derive(Debug)]
pub enum Retcode {
    OKAY,
    ERROR,
    NOMEMORY,
    READERROR,
    WRITEERROR,
    NOFILE,
    FILECREATEERROR,
    LPERROR,
    NOPROBLEM,
    INVALIDCALL,
    INVALIDDATA,
    INVALIDRESULT,
    PLUGINNOTFOUND,
    PARAMETERUNKNOWN,
    PARAMETERWRONGTYPE,
    PARAMETERWRONGVAL,
    KEYALREADYEXISTING,
    MAXDEPTHLEVEL,
    BRANCHERROR,
    NOTIMPLEMENTED,
}

impl Retcode {
    pub fn from_c_scip_retcode(val: c_api::SCIP_Retcode) -> Option<Self> {
        match val {
            c_api::SCIP_Retcode_SCIP_OKAY => Some(Retcode::OKAY),
            c_api::SCIP_Retcode_SCIP_ERROR => Some(Retcode::ERROR),
            c_api::SCIP_Retcode_SCIP_NOMEMORY => Some(Retcode::NOMEMORY),
            c_api::SCIP_Retcode_SCIP_READERROR => Some(Retcode::READERROR),
            c_api::SCIP_Retcode_SCIP_WRITEERROR => Some(Retcode::WRITEERROR),
            c_api::SCIP_Retcode_SCIP_NOFILE => Some(Retcode::NOFILE),
            c_api::SCIP_Retcode_SCIP_FILECREATEERROR => Some(Retcode::FILECREATEERROR),
            c_api::SCIP_Retcode_SCIP_LPERROR => Some(Retcode::LPERROR),
            c_api::SCIP_Retcode_SCIP_NOPROBLEM => Some(Retcode::NOPROBLEM),
            c_api::SCIP_Retcode_SCIP_INVALIDCALL => Some(Retcode::INVALIDCALL),
            c_api::SCIP_Retcode_SCIP_INVALIDDATA => Some(Retcode::INVALIDDATA),
            c_api::SCIP_Retcode_SCIP_INVALIDRESULT => Some(Retcode::INVALIDRESULT),
            c_api::SCIP_Retcode_SCIP_PLUGINNOTFOUND => Some(Retcode::PLUGINNOTFOUND),
            c_api::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => Some(Retcode::PARAMETERUNKNOWN),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => Some(Retcode::PARAMETERWRONGTYPE),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => Some(Retcode::PARAMETERWRONGVAL),
            c_api::SCIP_Retcode_SCIP_KEYALREADYEXISTING => Some(Retcode::KEYALREADYEXISTING),
            c_api::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => Some(Retcode::MAXDEPTHLEVEL),
            c_api::SCIP_Retcode_SCIP_BRANCHERROR => Some(Retcode::BRANCHERROR),
            c_api::SCIP_Retcode_SCIP_NOTIMPLEMENTED => Some(Retcode::NOTIMPLEMENTED),
            _ => None,
        }
    }
}
