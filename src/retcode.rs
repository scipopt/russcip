use crate::c_api;

#[derive(Debug)]
pub enum SCIPRetcode {
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

impl SCIPRetcode {
    pub fn from_c_scip_retcode(val: c_api::SCIP_Retcode) -> Option<Self> {
        match val {
            c_api::SCIP_Retcode_SCIP_OKAY => Some(SCIPRetcode::OKAY),
            c_api::SCIP_Retcode_SCIP_ERROR => Some(SCIPRetcode::ERROR),
            c_api::SCIP_Retcode_SCIP_NOMEMORY => Some(SCIPRetcode::NOMEMORY),
            c_api::SCIP_Retcode_SCIP_READERROR => Some(SCIPRetcode::READERROR),
            c_api::SCIP_Retcode_SCIP_WRITEERROR => Some(SCIPRetcode::WRITEERROR),
            c_api::SCIP_Retcode_SCIP_NOFILE => Some(SCIPRetcode::NOFILE),
            c_api::SCIP_Retcode_SCIP_FILECREATEERROR => Some(SCIPRetcode::FILECREATEERROR),
            c_api::SCIP_Retcode_SCIP_LPERROR => Some(SCIPRetcode::LPERROR),
            c_api::SCIP_Retcode_SCIP_NOPROBLEM => Some(SCIPRetcode::NOPROBLEM),
            c_api::SCIP_Retcode_SCIP_INVALIDCALL => Some(SCIPRetcode::INVALIDCALL),
            c_api::SCIP_Retcode_SCIP_INVALIDDATA => Some(SCIPRetcode::INVALIDDATA),
            c_api::SCIP_Retcode_SCIP_INVALIDRESULT => Some(SCIPRetcode::INVALIDRESULT),
            c_api::SCIP_Retcode_SCIP_PLUGINNOTFOUND => Some(SCIPRetcode::PLUGINNOTFOUND),
            c_api::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => Some(SCIPRetcode::PARAMETERUNKNOWN),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => Some(SCIPRetcode::PARAMETERWRONGTYPE),
            c_api::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => Some(SCIPRetcode::PARAMETERWRONGVAL),
            c_api::SCIP_Retcode_SCIP_KEYALREADYEXISTING => Some(SCIPRetcode::KEYALREADYEXISTING),
            c_api::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => Some(SCIPRetcode::MAXDEPTHLEVEL),
            c_api::SCIP_Retcode_SCIP_BRANCHERROR => Some(SCIPRetcode::BRANCHERROR),
            c_api::SCIP_Retcode_SCIP_NOTIMPLEMENTED => Some(SCIPRetcode::NOTIMPLEMENTED),
            _ => None,
        }
    }
}
