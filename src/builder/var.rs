use crate::builder::CanBeAddedToModel;
use crate::{Model, ModelWithProblem, ProblemCreated, VarType, Variable};

/// A builder for variables.
pub struct VarBuilder<'a> {
    name: Option<&'a str>,
    obj: f64,
    lb: f64,
    ub: f64,
    var_type: VarType,
}

/// Creates a new default `VarBuilder`.
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
    pub fn integer(mut self, lb: isize, ub: isize) -> Self {
        self.lb = lb as f64;
        self.ub = ub as f64;
        self.var_type = VarType::Integer;
        self
    }

    /// Sets the variable to be a binary variable.
    pub fn binary(mut self) -> Self {
        self.lb = 0.0;
        self.ub = 1.0;
        self.var_type = VarType::Binary;
        self
    }

    /// Sets the variable to be a continuous variable.
    pub fn continuous(mut self, lb: f64, ub: f64) -> Self {
        self.lb = lb;
        self.ub = ub;
        self.var_type = VarType::Continuous;
        self
    }

    /// Sets the variable to be an implicit integer variable.
    pub fn impl_int(mut self, lb: f64, ub: f64) -> Self {
        self.lb = lb;
        self.ub = ub;
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
        let var = VarBuilder::default()
            .name("x")
            .obj(1.0)
            .continuous(0.0, 1.0);

        assert_eq!(var.name, Some("x"));
        assert_eq!(var.obj, 1.0);
        assert_eq!(var.lb, 0.0);
        assert_eq!(var.ub, 1.0);
    }

    #[test]
    fn test_var_builder_add() {
        let mut model = Model::default().set_obj_sense(crate::ObjSense::Maximize);
        let var = var().name("x").obj(1.0).continuous(0.0, 1.0);

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
            var().name("1").obj(1.0).continuous(0.0, 1.0),
            var().name("2").obj(1.0).continuous(0.0, 1.0),
            var().name("3").obj(1.0).continuous(0.0, 1.0),
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
