use crate::ffi;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Retcode {
    Okay,
    Error,
    NoMemory,
    ReadError,
    WriteError,
    NoFile,
    FileCreateError,
    LpError,
    NoProblem,
    InvalidCall,
    InvalidData,
    InvalidResult,
    PluginNotFound,
    ParameterUnknown,
    ParameterWrongType,
    ParameterWrongVal,
    KeyAlreadyExisting,
    MaxDepthLevel,
    BranchError,
    NotImplemented,
    Unknown(ffi::SCIP_Retcode),
}

impl From<ffi::SCIP_Retcode> for Retcode {
    fn from(val: ffi::SCIP_Retcode) -> Self {
        match val {
            ffi::SCIP_Retcode_SCIP_OKAY => Retcode::Okay,
            ffi::SCIP_Retcode_SCIP_ERROR => Retcode::Error,
            ffi::SCIP_Retcode_SCIP_NOMEMORY => Retcode::NoMemory,
            ffi::SCIP_Retcode_SCIP_READERROR => Retcode::ReadError,
            ffi::SCIP_Retcode_SCIP_WRITEERROR => Retcode::WriteError,
            ffi::SCIP_Retcode_SCIP_NOFILE => Retcode::NoFile,
            ffi::SCIP_Retcode_SCIP_FILECREATEERROR => Retcode::FileCreateError,
            ffi::SCIP_Retcode_SCIP_LPERROR => Retcode::LpError,
            ffi::SCIP_Retcode_SCIP_NOPROBLEM => Retcode::NoProblem,
            ffi::SCIP_Retcode_SCIP_INVALIDCALL => Retcode::InvalidCall,
            ffi::SCIP_Retcode_SCIP_INVALIDDATA => Retcode::InvalidData,
            ffi::SCIP_Retcode_SCIP_INVALIDRESULT => Retcode::InvalidResult,
            ffi::SCIP_Retcode_SCIP_PLUGINNOTFOUND => Retcode::PluginNotFound,
            ffi::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => Retcode::ParameterUnknown,
            ffi::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => Retcode::ParameterWrongType,
            ffi::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => Retcode::ParameterWrongVal,
            ffi::SCIP_Retcode_SCIP_KEYALREADYEXISTING => Retcode::KeyAlreadyExisting,
            ffi::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => Retcode::MaxDepthLevel,
            ffi::SCIP_Retcode_SCIP_BRANCHERROR => Retcode::BranchError,
            ffi::SCIP_Retcode_SCIP_NOTIMPLEMENTED => Retcode::NotImplemented,
            val => Retcode::Unknown(val),
        }
    }
}
