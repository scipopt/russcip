use crate::builder::CanBeAddedToModel;
use crate::{Constraint, Model, ModelWithProblem, ProblemCreated, ProblemOrSolving, Variable};

/// A builder for creating constraints.
#[derive(Debug)]
pub struct ConsBuilder<'a> {
    lhs: f64,
    rhs: f64,
    name: Option<String>,
    coefs: Vec<(&'a Variable, f64)>,
}

/// Creates a new default `ConsBuilder`.
pub fn cons() -> ConsBuilder<'static> {
    ConsBuilder::default()
}

impl Default for ConsBuilder<'_> {
    fn default() -> Self {
        ConsBuilder {
            lhs: f64::NEG_INFINITY,
            rhs: f64::INFINITY,
            name: None,
            coefs: Vec::new(),
        }
    }
}

impl<'a> ConsBuilder<'a> {
    /// Creates a constraint of the form `expr <= val`.
    pub fn le(mut self, val: f64) -> Self {
        self.rhs = val;
        self.lhs = f64::NEG_INFINITY;
        self
    }

    /// Creates a constraint of the form `val <= expr`.
    pub fn ge(mut self, val: f64) -> Self {
        self.lhs = val;
        self.rhs = f64::INFINITY;
        self
    }

    /// Creates a constraint of the form `expr = val`.
    pub fn eq(mut self, val: f64) -> Self {
        self.lhs = val;
        self.rhs = val;
        self
    }

    /// Sets the name of the constraint.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Adds a coefficient to the constraint.
    pub fn coef(mut self, var: &'a Variable, coef: f64) -> Self {
        self.coefs.push((var, coef));
        self
    }
}

impl CanBeAddedToModel for ConsBuilder<'_> {
    type Return = Constraint;
    fn add(self, model: &mut Model<ProblemCreated>) -> Self::Return {
        let mut vars = Vec::new();
        let mut coefs = Vec::new();
        for (var, coef) in self.coefs {
            vars.push(var);
            coefs.push(coef);
        }

        let name = self.name.unwrap_or_else(|| {
            let n_cons = model.n_conss();
            format!("cons{}", n_cons)
        });
        model.add_cons(vars, &coefs, self.lhs, self.rhs, &name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::var::var;
    use crate::minimal_model;

    #[test]
    fn test_cons_builder() {
        let mut model = minimal_model().hide_output();
        let var = model.add(var().binary().obj(1.));
        let cons = cons().name("c".to_string()).eq(1.0).coef(&var, 1.0);

        assert_eq!(cons.name, Some("c".to_string()));
        assert_eq!(cons.lhs, 1.0);
        assert_eq!(cons.rhs, 1.0);
        assert_eq!(cons.coefs.len(), 1);
        assert_eq!(cons.coefs[0].1, 1.0);

        model.add(cons);

        assert_eq!(model.n_conss(), 1);
        let cons = &model.conss()[0];
        assert_eq!(cons.name(), "c");

        let solved = model.solve();

        assert_eq!(solved.status(), crate::Status::Optimal);
        assert_eq!(solved.obj_val(), 1.0);
    }
}
