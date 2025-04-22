use crate::builder::CanBeAddedToModel;
use crate::{
    Constraint, Model, ModelWithProblem, ProblemCreated, ProblemOrSolving, Solving, Variable,
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
    /// Coefficients of constraint
    pub(crate) coefs: Vec<(&'a Variable, f64)>,
    /// Modifiable flag of constraint
    pub(crate) modifiable: Option<bool>,
    /// Removable flag of constraint
    pub(crate) removable: Option<bool>,
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
            modifiable: None,
            removable: None,
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
        let cons = model.add_cons(vars, &coefs, self.lhs, self.rhs, &name);

        if let Some(modifiable) = self.modifiable {
            model.set_cons_modifiable(&cons, modifiable);
        }
        if let Some(removable) = self.removable {
            model.set_cons_removable(&cons, removable);
        }

        cons
    }
}

impl CanBeAddedToModel<Solving> for ConsBuilder<'_> {
    type Return = Constraint;
    fn add(self, model: &mut Model<Solving>) -> Self::Return {
        if self.modifiable.is_some() {
            panic!("cannot add a modifiable constraint during solving");
        }

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
        let cons3 = model.add(cb3);

        assert!(cons1.is_removable());
        assert!(!cons2.is_removable());
        assert!(!cons3.is_removable());

        let solved = model.solve();
        assert!(solved.cons_is_removable(&cons1));
        assert!(!solved.cons_is_removable(&cons2));
        assert!(!solved.cons_is_removable(&cons3));
    }
}
