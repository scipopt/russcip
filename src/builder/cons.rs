use crate::builder::CanBeAddedToModel;
use crate::{
    Constraint, Model, ModelWithProblem, ProblemCreated, ProblemOrSolving, Solving, Variable,
};

/// A builder for creating constraints.
#[derive(Debug)]
pub struct ConsBuilder<'a> {
    lhs: f64,
    rhs: f64,
    name: Option<&'a str>,
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
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Adds a coefficient to the constraint.
    pub fn coef(mut self, var: &'a Variable, coef: f64) -> Self {
        self.coefs.push((var, coef));
        self
    }

    /// Adds multiple coefficients to the constraint.
    pub fn coefs(mut self, var_refs: Vec<&'a Variable>, vals: Vec<f64>) -> Self {
        self.coefs.extend(var_refs.into_iter().zip(vals));
        self
    }
    /// Adds multiple coefficients to the constraint.
    pub fn expr<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a Variable, f64)>,
    {
        self.coefs.extend(iter);
        self
    }
}

impl CanBeAddedToModel<ProblemCreated> for ConsBuilder<'_> {
    type Return = Constraint;
    fn add(self, model: &mut Model<ProblemCreated>) -> Self::Return {
        let mut vars = Vec::new();
        let mut coefs = Vec::new();
        for (var, coef) in self.coefs {
            vars.push(var);
            coefs.push(coef);
        }

        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
            let n_cons = model.n_conss();
            format!("cons{}", n_cons)
        });
        model.add_cons(vars, &coefs, self.lhs, self.rhs, &name)
    }
}

impl CanBeAddedToModel<Solving> for ConsBuilder<'_> {
    type Return = Constraint;
    fn add(self, model: &mut Model<Solving>) -> Self::Return {
        let mut vars = Vec::new();
        let mut coefs = Vec::new();
        for (var, coef) in self.coefs {
            vars.push(var);
            coefs.push(coef);
        }

        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
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
        let var = model.add(var().bin().obj(1.));
        let cons = cons().name("c").eq(1.0).coef(&var, 1.0);

        assert_eq!(cons.name, Some("c"));
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

    #[test]
    fn test_cons_builder_expr() {
        let mut model = minimal_model().hide_output();
        let vars = [
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
        ];

        let cons = cons()
            .name("c")
            .eq(1.0)
            .expr(vars.iter().map(|var| (var, 1.0)));

        assert_eq!(cons.name, Some("c"));

        assert_eq!(cons.lhs, 1.0);
        assert_eq!(cons.rhs, 1.0);
        assert_eq!(cons.coefs.len(), 2);
        assert_eq!(cons.coefs[0].1, 1.0);
        assert_eq!(cons.coefs[1].1, 1.0);

        model.add(cons);

        assert_eq!(model.n_conss(), 1);
        let cons = &model.conss()[0];
        assert_eq!(cons.name(), "c");

        let solved = model.solve();

        assert_eq!(solved.status(), crate::Status::Optimal);
        assert_eq!(solved.obj_val(), 1.0);
    }
}
