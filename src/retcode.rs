use crate::ffi;

/// An enum representing the possible return codes from SCIP functions.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Retcode {
    /// Normal termination.
    Okay,
    /// Unspecified error.
    Error,
    /// Insufficient memory error.
    NoMemory,
    /// Read error.
    ReadError,
    /// Write error.
    WriteError,
    /// File not found error.
    NoFile,
    /// Cannot create file.
    FileCreateError,
    /// Error in LP solver.
    LpError,
    /// No problem exists.
    NoProblem,
    /// Method cannot be called at this time in solution process.
    InvalidCall,
    /// Error in input data.
    InvalidData,
    /// Method returned an invalid result code.
    InvalidResult,
    /// A required plugin was not found.
    PluginNotFound,
    /// The parameter with the given name was not found.
    ParameterUnknown,
    /// The parameter is not of the expected type.
    ParameterWrongType,
    /// The value is invalid for the given parameter.
    ParameterWrongVal,
    /// The given key is already existing in table.
    KeyAlreadyExisting,
    /// Maximal branching depth level exceeded.
    MaxDepthLevel,
    /// No branching could be created.
    BranchError,
    /// Function not implemented.
    NotImplemented,
    /// Any status code not specifically represented in this enum.
    Unknown(ffi::SCIP_Retcode),
}

impl From<ffi::SCIP_Retcode> for Retcode {
    /// Converts an `SCIP_Retcode` value to a `Retcode` enum variant.
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

impl From<Retcode> for ffi::SCIP_Retcode {
    /// Converts a `Retcode` enum variant to an `SCIP_Retcode` value.
    fn from(value: Retcode) -> Self {
        match value {
            Retcode::Okay => ffi::SCIP_Retcode_SCIP_OKAY,
            Retcode::Error => ffi::SCIP_Retcode_SCIP_ERROR,
            Retcode::NoMemory => ffi::SCIP_Retcode_SCIP_NOMEMORY,
            Retcode::ReadError => ffi::SCIP_Retcode_SCIP_READERROR,
            Retcode::WriteError => ffi::SCIP_Retcode_SCIP_WRITEERROR,
            Retcode::NoFile => ffi::SCIP_Retcode_SCIP_NOFILE,
            Retcode::FileCreateError => ffi::SCIP_Retcode_SCIP_FILECREATEERROR,
            Retcode::LpError => ffi::SCIP_Retcode_SCIP_LPERROR,
            Retcode::NoProblem => ffi::SCIP_Retcode_SCIP_NOPROBLEM,
            Retcode::InvalidCall => ffi::SCIP_Retcode_SCIP_INVALIDCALL,
            Retcode::InvalidData => ffi::SCIP_Retcode_SCIP_INVALIDDATA,
            Retcode::InvalidResult => ffi::SCIP_Retcode_SCIP_INVALIDRESULT,
            Retcode::PluginNotFound => ffi::SCIP_Retcode_SCIP_PLUGINNOTFOUND,
            Retcode::ParameterUnknown => ffi::SCIP_Retcode_SCIP_PARAMETERUNKNOWN,
            Retcode::ParameterWrongType => ffi::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE,
            Retcode::ParameterWrongVal => ffi::SCIP_Retcode_SCIP_PARAMETERWRONGVAL,
            Retcode::KeyAlreadyExisting => ffi::SCIP_Retcode_SCIP_KEYALREADYEXISTING,
            Retcode::MaxDepthLevel => ffi::SCIP_Retcode_SCIP_MAXDEPTHLEVEL,
            Retcode::BranchError => ffi::SCIP_Retcode_SCIP_BRANCHERROR,
            Retcode::NotImplemented => ffi::SCIP_Retcode_SCIP_NOTIMPLEMENTED,
            Retcode::Unknown(val) => val,
        }
    }
}
