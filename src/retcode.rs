use crate::ffi;

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
    pub fn from_c_scip_retcode(val: ffi::SCIP_Retcode) -> Option<Self> {
        match val {
            ffi::SCIP_Retcode_SCIP_OKAY => Some(Retcode::OKAY),
            ffi::SCIP_Retcode_SCIP_ERROR => Some(Retcode::ERROR),
            ffi::SCIP_Retcode_SCIP_NOMEMORY => Some(Retcode::NOMEMORY),
            ffi::SCIP_Retcode_SCIP_READERROR => Some(Retcode::READERROR),
            ffi::SCIP_Retcode_SCIP_WRITEERROR => Some(Retcode::WRITEERROR),
            ffi::SCIP_Retcode_SCIP_NOFILE => Some(Retcode::NOFILE),
            ffi::SCIP_Retcode_SCIP_FILECREATEERROR => Some(Retcode::FILECREATEERROR),
            ffi::SCIP_Retcode_SCIP_LPERROR => Some(Retcode::LPERROR),
            ffi::SCIP_Retcode_SCIP_NOPROBLEM => Some(Retcode::NOPROBLEM),
            ffi::SCIP_Retcode_SCIP_INVALIDCALL => Some(Retcode::INVALIDCALL),
            ffi::SCIP_Retcode_SCIP_INVALIDDATA => Some(Retcode::INVALIDDATA),
            ffi::SCIP_Retcode_SCIP_INVALIDRESULT => Some(Retcode::INVALIDRESULT),
            ffi::SCIP_Retcode_SCIP_PLUGINNOTFOUND => Some(Retcode::PLUGINNOTFOUND),
            ffi::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => Some(Retcode::PARAMETERUNKNOWN),
            ffi::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => Some(Retcode::PARAMETERWRONGTYPE),
            ffi::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => Some(Retcode::PARAMETERWRONGVAL),
            ffi::SCIP_Retcode_SCIP_KEYALREADYEXISTING => Some(Retcode::KEYALREADYEXISTING),
            ffi::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => Some(Retcode::MAXDEPTHLEVEL),
            ffi::SCIP_Retcode_SCIP_BRANCHERROR => Some(Retcode::BRANCHERROR),
            ffi::SCIP_Retcode_SCIP_NOTIMPLEMENTED => Some(Retcode::NOTIMPLEMENTED),
            _ => None,
        }
    }
}
