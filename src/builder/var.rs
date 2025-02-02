use crate::builder::CanBeAddedToModel;
use crate::{Model, ModelWithProblem, ProblemCreated, VarType, Variable};
use std::ops::RangeBounds;

/// A builder for variables.
pub struct VarBuilder<'a> {
    name: Option<&'a str>,
    obj: f64,
    lb: f64,
    ub: f64,
    var_type: VarType,
}

/// Creates a new default `VarBuilder`. It can be chained with other methods to set the properties of the variable.
///
/// # Example
///
/// ```rust
/// use russcip::prelude::*;
///
/// let integer_var = var().name("x").int(0..=10); // Integer variable with bounds [0, 10]
/// let binary_var = var().name("y").bin(); // Binary variable
/// let continuous_var = var().name("z").cont(0.0..); // Continuous variable with lower bound 0.0
///
/// let mut model = Model::default();
/// model.add(integer_var);
/// model.add(binary_var);
/// model.add(continuous_var);
/// ```
pub fn var<'a>() -> VarBuilder<'a> {
    VarBuilder::default()
}

impl Default for VarBuilder<'_> {
    fn default() -> Self {
        VarBuilder {
            name: None,
            obj: 0.0,
            lb: 0.0,
            ub: f64::INFINITY,
            var_type: VarType::Continuous,
        }
    }
}

impl<'a> VarBuilder<'a> {
    /// Sets the variable to be an integer variable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let var = var().int(0..=10); // Integer variable with bounds [0, 10]
    /// ```
    pub fn int<B: RangeBounds<isize>>(mut self, bounds: B) -> Self {
        match bounds.start_bound() {
            std::ops::Bound::Included(&lb) => self.lb = lb as f64,
            std::ops::Bound::Excluded(&lb) => self.lb = lb as f64 + 1.0,
            std::ops::Bound::Unbounded => {
                self.lb = f64::NEG_INFINITY;
            }
        }
        match bounds.end_bound() {
            std::ops::Bound::Included(&ub) => self.ub = ub as f64,
            std::ops::Bound::Excluded(&ub) => self.ub = ub as f64 - 1.0,
            std::ops::Bound::Unbounded => {
                self.ub = f64::INFINITY;
            }
        }
        self.var_type = VarType::Integer;
        self
    }

    /// Sets the variable to be a binary variable.
    ///
    /// # Example
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let var = var().bin(); // Binary variable
    /// ```
    pub fn bin(mut self) -> Self {
        self.lb = 0.0;
        self.ub = 1.0;
        self.var_type = VarType::Binary;
        self
    }

    /// Sets the variable to be a continuous variable.
    ///
    /// # Example
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let v1 = var().cont(0.0..); // Continuous variable with lower bound 0.0
    /// let v2 = var().cont(..=10.0); // Continuous variable with upper bound 10.0
    /// let v3 = var().cont(0.0..=10.0); // Continuous variable with bounds [0.0, 10.0]
    /// ```
    pub fn cont<B: RangeBounds<f64>>(mut self, bounds: B) -> Self {
        match bounds.start_bound() {
            std::ops::Bound::Included(&lb) => self.lb = lb,
            std::ops::Bound::Excluded(&lb) => self.lb = lb + 1e-6,
            std::ops::Bound::Unbounded => {
                self.lb = f64::NEG_INFINITY;
            }
        }
        match bounds.end_bound() {
            std::ops::Bound::Included(&ub) => self.ub = ub,
            std::ops::Bound::Excluded(&ub) => self.ub = ub - 1e-6,
            std::ops::Bound::Unbounded => {
                self.ub = f64::INFINITY;
            }
        }
        self.var_type = VarType::Continuous;
        self
    }

    /// Sets the variable to be an implicit integer variable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let var = var().impl_int(0..=10); // Implicit integer variable with bounds [0, 10]
    /// ```
    pub fn impl_int<B: RangeBounds<isize>>(mut self, bounds: B) -> Self {
        match bounds.start_bound() {
            std::ops::Bound::Included(&lb) => self.lb = lb as f64,
            std::ops::Bound::Excluded(&lb) => self.lb = (lb + 1) as f64,
            std::ops::Bound::Unbounded => {
                self.lb = f64::NEG_INFINITY;
            }
        }
        match bounds.end_bound() {
            std::ops::Bound::Included(&ub) => self.ub = ub as f64,
            std::ops::Bound::Excluded(&ub) => self.ub = (ub - 1) as f64,
            std::ops::Bound::Unbounded => {
                self.ub = f64::INFINITY;
            }
        }
        self.var_type = VarType::ImplInt;
        self
    }

    /// Sets the name of the variable.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the objective coefficient of the variable.
    pub fn obj(mut self, obj: f64) -> Self {
        self.obj = obj;
        self
    }
}

impl CanBeAddedToModel for VarBuilder<'_> {
    type Return = Variable;
    fn add(self, model: &mut Model<ProblemCreated>) -> Variable {
        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
            let n_vars = model.n_vars();
            format!("x{}", n_vars)
        });

        model.add_var(self.lb, self.ub, self.obj, &name, self.var_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_builder() {
        let var = VarBuilder::default().name("x").obj(1.0).cont(0.0..1.0);

        assert_eq!(var.name, Some("x"));
        assert_eq!(var.obj, 1.0);
        assert_eq!(var.lb, 0.0);
        assert_eq!(var.ub, 1.0);
    }

    #[test]
    fn test_var_builder_add() {
        let mut model = Model::default().set_obj_sense(crate::ObjSense::Maximize);
        let var = var().name("x").obj(1.0).cont(0.0..1.0);

        let var = model.add(var);

        assert_eq!(model.n_vars(), 1);
        assert_eq!(var.name(), "x");
        assert_eq!(var.obj(), 1.0);
        assert_eq!(var.lb(), 0.0);
        assert_eq!(var.ub(), 1.0);

        let solved = model.solve();
        assert_eq!(solved.status(), crate::Status::Optimal);
        assert_eq!(solved.obj_val(), 1.0);
    }

    #[test]
    fn test_var_add_all() {
        let mut model = Model::default().set_obj_sense(crate::ObjSense::Maximize);
        let vars = vec![
            var().name("1").obj(1.0).cont(0.0..1.0),
            var().name("2").obj(1.0).cont(0.0..1.0),
            var().name("3").obj(1.0).cont(0.0..1.0),
        ];

        let vars = model.add(vars);
        for (i, var) in vars.iter().enumerate() {
            assert_eq!(var.name(), (i + 1).to_string());
            assert_eq!(var.obj(), 1.0);
            assert_eq!(var.lb(), 0.0);
            assert_eq!(var.ub(), 1.0);
        }

        let solved = model.solve();
        assert_eq!(solved.status(), crate::Status::Optimal);
        assert_eq!(solved.obj_val(), 3.0);
    }
}
