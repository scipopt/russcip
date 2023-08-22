use crate::ffi;
use core::panic;
use std::any::Any;

/// A type alias for a variable ID.
pub type VarId = usize;

/// A wrapper for a mutable reference to a SCIP variable.
#[derive(Debug)]
pub struct Variable {
    pub(crate) raw: *mut ffi::SCIP_VAR,
    pub(crate) data: Option<Box<dyn Any>>,
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Variable {
    #[cfg(feature = "raw")]
    /// Returns a raw pointer to the underlying `ffi::SCIP_VAR` struct.
    pub fn inner(&self) -> *mut ffi::SCIP_VAR {
        self.raw
    }

    /// Returns the index of the variable.
    pub fn index(&self) -> usize {
        let id = unsafe { ffi::SCIPvarGetIndex(self.raw) };
        assert!(id >= 0);
        id as usize
    }

    /// Returns the name of the variable.
    pub fn name(&self) -> String {
        let name = unsafe { ffi::SCIPvarGetName(self.raw) };
        let name = unsafe { std::ffi::CStr::from_ptr(name) };
        name.to_str().unwrap().to_string()
    }

    /// Returns the objective coefficient of the variable.
    pub fn obj(&self) -> f64 {
        unsafe { ffi::SCIPvarGetObj(self.raw) }
    }

    /// Returns the lower bound of the variable.
    pub fn lb(&self) -> f64 {
        unsafe { ffi::SCIPvarGetLbLocal(self.raw) }
    }

    /// Returns the upper bound of the variable.
    pub fn ub(&self) -> f64 {
        unsafe { ffi::SCIPvarGetUbLocal(self.raw) }
    }

    /// Returns the type of the variable.
    pub fn var_type(&self) -> VarType {
        let var_type = unsafe { ffi::SCIPvarGetType(self.raw) };
        var_type.into()
    }

    /// Returns the status of the variable.
    pub fn status(&self) -> VarStatus {
        let status = unsafe { ffi::SCIPvarGetStatus(self.raw) };
        status.into()
    }

    /// Attaches some data to the variable.
    pub fn set_data<T: Any>(&mut self, data: T) {
        self.data = Some(Box::new(data));
    }

    /// Returns a reference to the attached data.
    pub fn get_data<T: Any>(&self) -> Option<&T> {
        self.data.as_ref().and_then(|data| data.downcast_ref::<T>())
    }
}

/// The type of a variable in an optimization problem.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    /// The variable is a continuous variable.
    Continuous,
    /// The variable is an integer variable.
    Integer,
    /// The variable is a binary variable.
    Binary,
    /// The variable is an implicit integer variable.
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarStatus {
    /// The variable is an original variable in the problem.
    Original,
    /// The variable is a loose variable in the problem.
    Loose,
    /// The variable is a column variable in the problem.
    Column,
    /// The variable is a fixed variable in the problem.
    Fixed,
    /// The variable is an aggregated variable in the problem.
    Aggregated,
    /// The variable is a multi-aggregated variable in the problem.
    MultiAggregated,
    /// The variable is a negated variable in the problem.
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
    use crate::Model;

    #[test]
    fn var_data() {
        let mut model = Model::new().include_default_plugins().create_prob("test");
        let var = model.add_var(0.0, 1.0, 2.0, "x", VarType::ImplInt);

        assert_eq!(var.borrow().index(), 0);
        assert_eq!(var.borrow().lb(), 0.0);
        assert_eq!(var.borrow().ub(), 1.0);
        assert_eq!(var.borrow().obj(), 2.0);
        assert_eq!(var.borrow().name(), "x");
        assert_eq!(var.borrow().var_type(), VarType::ImplInt);
        assert_eq!(var.borrow().status(), VarStatus::Original);

        #[cfg(feature = "raw")]
        assert!(!var.borrow().inner().is_null());
    }

    #[test]
    fn attach_data() {
        let mut model = Model::new().include_default_plugins().create_prob("test");
        let mut var = model.add_var(0.0, 1.0, 2.0, "x", VarType::ImplInt);
        var.borrow_mut().set_data(42);
        assert_eq!(var.borrow().get_data::<i32>().unwrap(), &42);
    }
}
