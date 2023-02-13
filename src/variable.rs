use core::panic;
use std::{ffi::CString, mem};

use crate::{ffi, model::Model, scip_call};

pub struct Variable<'a> {
    pub(crate) raw: *mut ffi::SCIP_VAR,
    pub(crate) model: &'a Model,
}

impl<'a> Variable<'a> {
    pub fn new(model: &'a Model,
    lb: f64,
    ub: f64,
    obj: f64,
    name: &str,
    var_type: VarType) -> Self {
        let name = CString::new(name).unwrap();
        let mut var_ptr: *mut ffi::SCIP_VAR = unsafe { mem::zeroed() };
        scip_call! { ffi::SCIPcreateVarBasic(
            model.scip,
            &mut var_ptr,
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        scip_call! { ffi::SCIPaddVar(model.scip, var_ptr) };
        Variable {
            raw: var_ptr,
            model
        }
    }

    pub fn get_index(&self) -> usize {
        let id = unsafe { ffi::SCIPvarGetIndex(self.raw) }; 
        if id < 0 {
            panic!("Variable index is negative");
        } else {
            id as usize
        }
    }
    
    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::SCIPvarGetName(self.raw) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    pub fn get_obj(&self) -> f64 {
        unsafe { ffi::SCIPvarGetObj(self.raw) }
    }

    pub fn get_lb(&self) -> f64 {
        unsafe { ffi::SCIPvarGetLbLocal(self.raw) }
    }

    pub fn get_ub(&self) -> f64 {
        unsafe { ffi::SCIPvarGetUbLocal(self.raw) }
    }

    pub fn get_type(&self) -> VarType {
        let var_type = unsafe { ffi::SCIPvarGetType(self.raw) };
        var_type.into()
    }
}

impl<'a> Drop for Variable<'a> {
    fn drop(&mut self) {
        unsafe { ffi::SCIPreleaseVar(self.model.scip, &mut self.raw) };
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_data() {
        let mut model = Model::new();
        model.include_default_plugins();
        model.create_prob("test");
        let var = Variable::new(&model, 0.0, 1.0, 2.0, "x", VarType::Binary);
        assert_eq!(var.get_index(), 0);
        assert_eq!(var.get_lb(), 0.0);
        assert_eq!(var.get_ub(), 1.0);
        assert_eq!(var.get_obj(), 2.0);
        assert_eq!(var.get_name(), "x");
        assert_eq!(var.get_type(), VarType::Binary);
    }
}

