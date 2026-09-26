use crate::builder::CanBeAddedToModel;
use crate::{Model, ModelWithProblem, ProblemCreated, Solving, VarType, Variable};
use std::ops::RangeBounds;

/// A builder for variables. It can be easily created using the `var` function.
///
/// # Example
///
/// ```rust
/// use russcip::prelude::*;
///
/// let integer_var = var().name("x").int(0..=10); // Integer variable with bounds [0, 10]
/// let binary_var = var().name("y").bin(); // Binary variable
/// let continuous_var = var().name("z").cont(0.0..); // Continuous variable with lower bound 0.0
/// let semi_continuous_var = var().name("s").cont(2.0..=10.0).semi_cont();
/// let semi_integer_var = var().name("t").int(2..=10).semi_int();
/// ```
pub struct VarBuilder<'a> {
    name: Option<&'a str>,
    obj: f64,
    lb: f64,
    ub: f64,
    var_type: VarType,
    semi_kind: Option<SemiVarKind>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SemiVarKind {
    Continuous,
    Integer,
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
/// let semi_continuous_var = var().name("s").cont(2.0..=10.0).semi_cont();
/// let semi_integer_var = var().name("t").int(2..=10).semi_int();
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
            semi_kind: None,
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
        self.semi_kind = None;
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
        self.semi_kind = None;
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
        self.semi_kind = None;
        self
    }

    /// Sets the variable to be semi-continuous.
    ///
    /// Set the nonzero domain first with [`cont`](Self::cont); calling this
    /// additionally allows the variable to be zero. The lower bound must be
    /// finite and strictly positive. The SCIP variable is created with lower
    /// bound zero and a bound-disjunction constraint enforces the nonzero range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let var = var().cont(2.0..=10.0).semi_cont();
    /// ```
    pub fn semi_cont(mut self) -> Self {
        self.var_type = VarType::Continuous;
        self.semi_kind = Some(SemiVarKind::Continuous);
        self
    }

    /// Sets the variable to be semi-integer.
    ///
    /// Set the nonzero domain first with [`int`](Self::int); calling this
    /// additionally allows the variable to be zero. The lower bound must be
    /// strictly positive. The SCIP variable is created with lower bound zero
    /// and a bound-disjunction constraint enforces the nonzero range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use russcip::prelude::*;
    ///
    /// let var = var().int(2..=10).semi_int();
    /// ```
    pub fn semi_int(mut self) -> Self {
        self.var_type = VarType::Integer;
        self.semi_kind = Some(SemiVarKind::Integer);
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
        self.semi_kind = None;
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

impl CanBeAddedToModel<ProblemCreated> for VarBuilder<'_> {
    type Return = Variable;
    fn add(self, model: &mut Model<ProblemCreated>) -> Variable {
        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
            let n_vars = model.n_vars();
            format!("x{n_vars}")
        });

        match self.semi_kind {
            Some(SemiVarKind::Continuous) => {
                model.add_semi_continuous_var(self.lb, self.ub, self.obj, &name)
            }
            Some(SemiVarKind::Integer) => {
                model.add_semi_integer_var(self.lb, self.ub, self.obj, &name)
            }
            None => model.add_var(self.lb, self.ub, self.obj, &name, self.var_type),
        }
    }
}

impl CanBeAddedToModel<Solving> for VarBuilder<'_> {
    type Return = Variable;
    fn add(self, model: &mut Model<Solving>) -> Variable {
        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
            let n_vars = model.n_vars();
            format!("x{n_vars}")
        });

        match self.semi_kind {
            Some(SemiVarKind::Continuous) => {
                model.add_semi_continuous_var(self.lb, self.ub, self.obj, &name)
            }
            Some(SemiVarKind::Integer) => {
                model.add_semi_integer_var(self.lb, self.ub, self.obj, &name)
            }
            None => model.add_var(self.lb, self.ub, self.obj, &name, self.var_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_builder() {
        let var = VarBuilder::default().name("x").obj(1.0).cont(0.0..=1.0);

        assert_eq!(var.name, Some("x"));
        assert_eq!(var.obj, 1.0);
        assert_eq!(var.lb, 0.0);
        assert_eq!(var.ub, 1.0);
    }

    #[test]
    fn test_semi_var_builder() {
        let semi_cont = var().cont(2.0..=10.0).semi_cont();
        assert_eq!(semi_cont.lb, 2.0);
        assert_eq!(semi_cont.ub, 10.0);
        assert_eq!(semi_cont.var_type, VarType::Continuous);
        assert_eq!(semi_cont.semi_kind, Some(SemiVarKind::Continuous));

        let semi_int = var().int(2..10).semi_int();
        assert_eq!(semi_int.lb, 2.0);
        assert_eq!(semi_int.ub, 9.0);
        assert_eq!(semi_int.var_type, VarType::Integer);
        assert_eq!(semi_int.semi_kind, Some(SemiVarKind::Integer));
    }

    #[test]
    fn changing_variable_type_clears_semi_kind() {
        let mut model = Model::default();
        let continuous = model.add(var().int(2..=5).semi_int().cont(1.0..=5.0));
        let integer = model.add(var().cont(2.0..=5.0).semi_cont().int(1..=5));

        assert_eq!(continuous.lb(), 1.0);
        assert_eq!(continuous.var_type(), VarType::Continuous);
        assert_eq!(integer.lb(), 1.0);
        assert_eq!(integer.var_type(), VarType::Integer);
        assert_eq!(model.n_conss(), 0);
    }

    #[test]
    fn test_var_builder_add() {
        let mut model = Model::default().set_obj_sense(crate::ObjSense::Maximize);
        let var = var().name("x").obj(1.0).cont(0.0..=1.0);

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
            var().name("1").obj(1.0).cont(0.0..=1.0),
            var().name("2").obj(1.0).cont(0.0..=1.0),
            var().name("3").obj(1.0).cont(0.0..=1.0),
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
