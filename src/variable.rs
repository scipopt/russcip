use crate::c_api;

pub struct Variable {
    pub(crate) ptr: *mut c_api::SCIP_VAR,
    pub(crate) scip_ptr: *mut c_api::SCIP,
}

impl Variable {
    pub fn new(scip_ptr: *mut c_api::SCIP, scip_var: *mut c_api::SCIP_VAR) -> Self {
        Variable { scip_ptr, ptr: scip_var }
    }

    pub fn get_name(&self) -> String {
        let name = unsafe { c_api::SCIPvarGetName(self.ptr) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    pub fn get_obj(&self) -> f64 {
        unsafe { c_api::SCIPvarGetObj(self.ptr) }
    }

    pub fn get_lb(&self) -> f64 {
        unsafe { c_api::SCIPvarGetLbLocal(self.ptr) }
    }

    pub fn get_ub(&self) -> f64 {
        unsafe { c_api::SCIPvarGetUbLocal(self.ptr) }
    }

    pub fn get_type(&self) -> VarType {
        let var_type = unsafe { c_api::SCIPvarGetType(self.ptr) };
        var_type.into()
    }
}

impl Drop for Variable {
    fn drop(&mut self) {
        unsafe { c_api::SCIPreleaseVar(self.scip_ptr, &mut self.ptr) };
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Continuous,
    Integer,
    Binary,
    ImplInt,
}

impl Into<VarType> for c_api::SCIP_Vartype {
    fn into(self) -> VarType {
        match self {
            c_api::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS => VarType::Continuous,
            c_api::SCIP_Vartype_SCIP_VARTYPE_INTEGER => VarType::Integer,
            c_api::SCIP_Vartype_SCIP_VARTYPE_BINARY => VarType::Binary,
            c_api::SCIP_Vartype_SCIP_VARTYPE_IMPLINT => VarType::ImplInt,
            _ => panic!("Unknown VarType {:?}", self),
        }
    }
}

impl From<VarType> for c_api::SCIP_Vartype {
    fn from(var_type: VarType) -> Self {
        match var_type {
            VarType::Continuous => c_api::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
            VarType::Integer => c_api::SCIP_Vartype_SCIP_VARTYPE_INTEGER,
            VarType::Binary => c_api::SCIP_Vartype_SCIP_VARTYPE_BINARY,
            VarType::ImplInt => c_api::SCIP_Vartype_SCIP_VARTYPE_IMPLINT,
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

impl Into<VarStatus> for c_api::SCIP_Varstatus {
    fn into(self) -> VarStatus {
        match self {
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_ORIGINAL => VarStatus::Original,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_LOOSE => VarStatus::Loose,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_COLUMN => VarStatus::Column,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_FIXED => VarStatus::Fixed,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_AGGREGATED => VarStatus::Aggregated,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_MULTAGGR => VarStatus::MultiAggregated,
            c_api::SCIP_Varstatus_SCIP_VARSTATUS_NEGATED => VarStatus::NegatedVar,
            _ => panic!("Unhandled SCIP variable status {:?}", self),
        }
    }
}

// TODO: implement parameter overloading for variable to use SCIP's tolerance values
