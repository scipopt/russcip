use core::panic;

use crate::ffi;

pub struct Variable {
    pub(crate) ptr: *mut ffi::SCIP_VAR,
    pub(crate) scip_ptr: *mut ffi::SCIP,
}

impl Variable {
    pub fn new(scip_ptr: *mut ffi::SCIP, scip_var: *mut ffi::SCIP_VAR) -> Self {
        Variable {
            scip_ptr,
            ptr: scip_var,
        }
    }

    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::SCIPvarGetName(self.ptr) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    pub fn get_obj(&self) -> f64 {
        unsafe { ffi::SCIPvarGetObj(self.ptr) }
    }

    pub fn get_lb(&self) -> f64 {
        unsafe { ffi::SCIPvarGetLbLocal(self.ptr) }
    }

    pub fn get_ub(&self) -> f64 {
        unsafe { ffi::SCIPvarGetUbLocal(self.ptr) }
    }

    pub fn get_type(&self) -> VarType {
        let var_type = unsafe { ffi::SCIPvarGetType(self.ptr) };
        var_type.into()
    }
}

impl Drop for Variable {
    fn drop(&mut self) {
        unsafe { ffi::SCIPreleaseVar(self.scip_ptr, &mut self.ptr) };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Continuous,
    Integer,
    Binary,
    ImplInt,
}

impl Into<VarType> for ffi::SCIP_Vartype {
    fn into(self) -> VarType {
        match self {
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS => VarType::Continuous,
            ffi::SCIP_Vartype_SCIP_VARTYPE_INTEGER => VarType::Integer,
            ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY => VarType::Binary,
            ffi::SCIP_Vartype_SCIP_VARTYPE_IMPLINT => VarType::ImplInt,
            _ => panic!("Unknown VarType {:?}", self),
        }
    }
}

impl From<VarType> for ffi::SCIP_Vartype {
    fn from(var_type: VarType) -> Self {
        match var_type {
            VarType::Continuous => ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
            VarType::Integer => ffi::SCIP_Vartype_SCIP_VARTYPE_INTEGER,
            VarType::Binary => ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY,
            VarType::ImplInt => ffi::SCIP_Vartype_SCIP_VARTYPE_IMPLINT,
        }
    }
}

pub enum VarStatus {
    Original,
    Loose,
    Column,
    Fixed,
    Aggregated,
    MultiAggregated,
    NegatedVar,
}

impl Into<VarStatus> for ffi::SCIP_Varstatus {
    fn into(self) -> VarStatus {
        match self {
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_ORIGINAL => VarStatus::Original,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_LOOSE => VarStatus::Loose,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_COLUMN => VarStatus::Column,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_FIXED => VarStatus::Fixed,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_AGGREGATED => VarStatus::Aggregated,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_MULTAGGR => VarStatus::MultiAggregated,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_NEGATED => VarStatus::NegatedVar,
            _ => panic!("Unhandled SCIP variable status {:?}", self),
        }
    }
}

// TODO: implement parameter overloading for variable to use SCIP's tolerance values
