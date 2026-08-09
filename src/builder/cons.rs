use crate::builder::CanBeAddedToModel;
use crate::expr::{Expr, split_constant};
use crate::{
    Constraint, Model, ModelStageProblemOrSolving, ModelStageWithProblem, ModelWithProblem,
    ProblemOrSolving, Variable,
};

/// A builder for creating constraints.
#[derive(Debug)]
pub struct ConsBuilder<'a> {
    /// Left-hand side of constraint
    pub(crate) lhs: f64,
    /// Right-hand side of constraint
    pub(crate) rhs: f64,
    /// (Optional) name of constraint
    pub(crate) name: Option<&'a str>,
    /// Coefficients of constraint, used when the body is linear
    pub(crate) coefs: Vec<(&'a Variable, f64)>,
    /// A (possibly nonlinear) expression body. When set, this is what the
    /// constraint is built from and `coefs` is ignored — the builder records
    /// which kind it is rather than inferring it later.
    pub(crate) ex: Option<Expr>,
    /// Modifiable flag of constraint
    pub(crate) modifiable: Option<bool>,
    /// Removable flag of constraint
    pub(crate) removable: Option<bool>,
    /// Separated flag of constraint
    pub(crate) separated: Option<bool>,
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
            ex: None,
            modifiable: None,
            removable: None,
            separated: None,
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

    /// Uses a (possibly nonlinear) [`Expr`] as the constraint body.
    ///
    /// Any coefficients previously added with [`ConsBuilder::coef`] are
    /// ignored. A body that turns out to be linear becomes a genuine linear
    /// constraint rather than a nonlinear one presolve has to upgrade.
    ///
    /// ```
    /// # use russcip::prelude::*;
    /// let mut model = Model::default().maximize().hide_output();
    /// let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
    ///
    /// model.add(cons().expression(Expr::pow(Expr::var(&x), 2.0)).le(16.0));
    ///
    /// let solved = model.solve();
    /// assert!((solved.obj_val() - 4.0).abs() < 1e-6);
    /// ```
    pub fn expression(mut self, ex: Expr) -> Self {
        self.ex = Some(ex);
        self
    }

    /// Sets both sides at once, for a two-sided constraint `lhs <= body <= rhs`.
    pub fn bounds(mut self, lhs: f64, rhs: f64) -> Self {
        self.lhs = lhs;
        self.rhs = rhs;
        self
    }

    /// Sets the modifiable flag of the constraint
    pub fn modifiable(mut self, modifiable: bool) -> Self {
        self.modifiable = Some(modifiable);
        self
    }

    /// Sets the removable flag of the constraint
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = Some(removable);
        self
    }

    /// Sets whether the constraint should be separated during LP processing
    pub fn separated(mut self, separate: bool) -> Self {
        self.separated = Some(separate);
        self
    }
}

impl<S> CanBeAddedToModel<S> for ConsBuilder<'_>
where
    S: ModelStageProblemOrSolving + ModelStageWithProblem,
{
    type Return = Constraint;
    fn add(self, model: &mut Model<S>) -> Self::Return {
        let mut vars = Vec::new();
        let mut coefs = Vec::new();
        for (var, coef) in self.coefs {
            vars.push(var);
            coefs.push(coef);
        }

        let name = self.name.map(|s| s.to_string()).unwrap_or_else(|| {
            let n_cons = model.n_conss();
            format!("cons{n_cons}")
        });

        // Shifting a body constant into the bounds: `x^2 - 16 <= 0` is
        // `x^2 <= 16`.
        fn shift(bound: f64, k: f64) -> f64 {
            if bound.is_finite() { bound - k } else { bound }
        }

        let cons = match self.ex {
            // A linear body becomes a linear constraint, not a nonlinear one
            // that presolve has to upgrade.
            Some(ref ex) if ex.as_linear().is_some() => {
                let (terms, k) = ex.as_linear().expect("checked above");
                let lhs = shift(self.lhs, k);
                let rhs = shift(self.rhs, k);
                let vars: Vec<&Variable> = terms.iter().map(|(v, _)| v).collect();
                let coefs: Vec<f64> = terms.iter().map(|(_, c)| *c).collect();
                model.add_cons(vars, &coefs, lhs, rhs, &name)
            }
            Some(ex) => {
                // A body written as `x^2 - 16` with a bound of 0 is the same
                // constraint as `x^2 <= 16`, so move any constant out of the
                // expression and into the bounds.
                let (ex, k) = split_constant(ex);
                let lhs = shift(self.lhs, k);
                let rhs = shift(self.rhs, k);
                let built = model
                    .build_expr(&ex)
                    .expect("failed to build constraint expression");
                model.add_cons_nonlinear(&built, lhs, rhs, &name)
            }
            None => model.add_cons(vars, &coefs, self.lhs, self.rhs, &name),
        };

        if let Some(modifiable) = self.modifiable {
            model.set_cons_modifiable(&cons, modifiable);
        }
        if let Some(removable) = self.removable {
            model.set_cons_removable(&cons, removable);
        }
        if let Some(separate) = self.separated {
            model.set_cons_separated(&cons, separate);
        }

        cons
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

    #[test]
    fn test_cons_builder_modifiable() {
        let mut model = minimal_model().hide_output();
        let vars = [
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
        ];

        let cb1 = cons()
            .name("c1")
            .le(2.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .modifiable(true);

        let cb2 = cons()
            .name("c2")
            .ge(1.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .modifiable(false);

        let cb3 = cons().name("c3").ge(1.0).coef(&vars[0], 1.0);

        assert_eq!(cb1.modifiable, Some(true));
        assert_eq!(cb2.modifiable, Some(false));
        assert_eq!(cb3.modifiable, None);

        let cons1 = model.add(cb1);
        let cons2 = model.add(cb2);
        let cons3 = model.add(cb3);

        assert!(cons1.is_modifiable());
        assert!(!cons2.is_modifiable());
        assert!(!cons3.is_modifiable());

        let solved = model.solve();
        assert!(solved.cons_is_modifiable(&cons1));
        assert!(!solved.cons_is_modifiable(&cons2));
        assert!(!solved.cons_is_modifiable(&cons3));
    }

    #[test]
    fn test_cons_builder_removable() {
        let mut model = minimal_model().hide_output();
        let vars = [
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
        ];

        let cb1 = cons()
            .name("c1")
            .le(2.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .removable(true);

        let cb2 = cons()
            .name("c2")
            .ge(1.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .removable(false);

        let cb3 = cons().name("c3").ge(1.0).coef(&vars[0], 1.0);

        assert_eq!(cb1.removable, Some(true));
        assert_eq!(cb2.removable, Some(false));
        assert_eq!(cb3.removable, None);

        let cons1 = model.add(cb1);
        let cons2 = model.add(cb2);

        assert!(cons1.is_removable());
        assert!(!cons2.is_removable());

        let solved = model.solve();
        assert!(solved.cons_is_removable(&cons1));
        assert!(!solved.cons_is_removable(&cons2));
    }

    #[test]
    fn test_cons_builder_separated() {
        let mut model = minimal_model().hide_output();
        let vars = [
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
            model.add(var().bin().obj(1.)),
        ];

        let cb1 = cons()
            .name("c1")
            .le(2.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .separated(true);

        let cb2 = cons()
            .name("c2")
            .ge(1.0)
            .expr(vars.iter().map(|var| (var, 1.0)))
            .separated(false);

        let cb3 = cons().name("c3").ge(1.0).coef(&vars[0], 1.0);

        assert_eq!(cb1.separated, Some(true));
        assert_eq!(cb2.separated, Some(false));
        assert_eq!(cb3.separated, None);

        let cons1 = model.add(cb1);
        let cons2 = model.add(cb2);

        assert!(cons1.is_separated());
        assert!(!cons2.is_separated());

        let solved = model.solve();
        assert!(solved.cons_is_separated(&cons1));
        assert!(!solved.cons_is_separated(&cons2));
    }
}
