use crate::{ffi, retcode::Retcode, scip_call};
use core::panic;
use std::{ffi::CString, mem::MaybeUninit};

pub type VarId = usize;
#[derive(Debug)]
pub struct Variable {
    pub(crate) raw: *mut ffi::SCIP_VAR,
}

impl Variable {
    pub(crate) fn new(
        scip_ptr: *mut ffi::SCIP,
        lb: f64,
        ub: f64,
        obj: f64,
        name: &str,
        var_type: VarType,
    ) -> Result<Self, Retcode> {
        let name = CString::new(name).unwrap();
        let mut var_ptr = MaybeUninit::uninit();
        scip_call! { ffi::SCIPcreateVarBasic(
            scip_ptr,
            var_ptr.as_mut_ptr(),
            name.as_ptr(),
            lb,
            ub,
            obj,
            var_type.into(),
        ) };
        let var_ptr = unsafe { var_ptr.assume_init() };
        scip_call! { ffi::SCIPaddVar(scip_ptr, var_ptr) };
        Ok(Variable { raw: var_ptr })
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Continuous,
    Integer,
    Binary,
    ImplInt,
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

impl From<ffi::SCIP_Vartype> for VarType {
    fn from(var_type: ffi::SCIP_Vartype) -> Self {
        match var_type {
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS => VarType::Continuous,
            ffi::SCIP_Vartype_SCIP_VARTYPE_INTEGER => VarType::Integer,
            ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY => VarType::Binary,
            ffi::SCIP_Vartype_SCIP_VARTYPE_IMPLINT => VarType::ImplInt,
            _ => panic!("Unknown VarType {:?}", var_type),
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

impl From<u32> for VarStatus {
    fn from(status: u32) -> Self {
        match status {
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_ORIGINAL => VarStatus::Original,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_LOOSE => VarStatus::Loose,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_COLUMN => VarStatus::Column,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_FIXED => VarStatus::Fixed,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_AGGREGATED => VarStatus::Aggregated,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_MULTAGGR => VarStatus::MultiAggregated,
            ffi::SCIP_Varstatus_SCIP_VARSTATUS_NEGATED => VarStatus::NegatedVar,
            _ => panic!("Unhandled SCIP variable status {:?}", status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, ModelWithProblem};
    use crate::retcode::Retcode;

    #[test]
    fn var_data() -> Result<(), Retcode> {
        let mut model = Model::new().include_default_plugins().create_prob("test");
        let var_id = model.add_var(0.0, 1.0, 2.0, "x", VarType::Binary);
        let var = model.get_var(var_id).unwrap();
        assert_eq!(var.get_index(), 0);
        assert_eq!(var.get_lb(), 0.0);
        assert_eq!(var.get_ub(), 1.0);
        assert_eq!(var.get_obj(), 2.0);
        assert_eq!(var.get_name(), "x");
        assert_eq!(var.get_type(), VarType::Binary);
        Ok(())
    }
}
