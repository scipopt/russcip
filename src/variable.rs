use crate::ffi;
use core::panic;

/// A type alias for a variable ID.
pub type VarId = usize;

/// A wrapper for a mutable reference to a SCIP variable.
#[derive(Debug)]
pub struct Variable {
    pub(crate) raw: *mut ffi::SCIP_VAR,
}

impl Variable {
    #[cfg(feature = "raw")]
    /// Returns a raw pointer to the underlying `ffi::SCIP_VAR` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_VAR {
        self.raw
    }

    /// Returns the index of the variable.
    pub fn get_index(&self) -> usize {
        let id = unsafe { ffi::SCIPvarGetIndex(self.raw) };
        if id < 0 {
            panic!("Variable index is negative");
        } else {
            id as usize
        }
    }

    /// Returns the name of the variable.
    pub fn get_name(&self) -> String {
        let name = unsafe { ffi::SCIPvarGetName(self.raw) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    /// Returns the objective coefficient of the variable.
    pub fn get_obj(&self) -> f64 {
        unsafe { ffi::SCIPvarGetObj(self.raw) }
    }

    /// Returns the lower bound of the variable.
    pub fn get_lb(&self) -> f64 {
        unsafe { ffi::SCIPvarGetLbLocal(self.raw) }
    }

    /// Returns the upper bound of the variable.
    pub fn get_ub(&self) -> f64 {
        unsafe { ffi::SCIPvarGetUbLocal(self.raw) }
    }

    /// Returns the type of the variable.
    pub fn get_type(&self) -> VarType {
        let var_type = unsafe { ffi::SCIPvarGetType(self.raw) };
        var_type.into()
    }
}

/// An enum representing the type of a SCIP variable.
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

/// An enum representing the status of a SCIP variable.
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
    use crate::model::Model;
    use crate::retcode::Retcode;

    #[test]
    fn var_data() -> Result<(), Retcode> {
        let mut model = Model::new().include_default_plugins().create_prob("test");
        let var = model.add_var(0.0, 1.0, 2.0, "x", VarType::Binary);
        assert_eq!(var.get_index(), 0);
        assert_eq!(var.get_lb(), 0.0);
        assert_eq!(var.get_ub(), 1.0);
        assert_eq!(var.get_obj(), 2.0);
        assert_eq!(var.get_name(), "x");
        assert_eq!(var.get_type(), VarType::Binary);
        Ok(())
    }
}