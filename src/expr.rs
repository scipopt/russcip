use crate::Variable;
use std::fmt;

/// A description of a nonlinear expression, built before any SCIP objects exist.
///
/// `Expr` is a plain Rust tree: constructing it allocates nothing in SCIP and
/// cannot fail. It is turned into a real [`ScipExpr`](crate::ScipExpr) — and therefore
/// into `SCIP_EXPR` objects — by
/// [`build_expr`](crate::ProblemOrSolving::build_expr).
///
/// Sums and products are **n-ary**, mirroring SCIP's own expression model
/// (`SCIPcreateExprSum` takes coefficients and a constant; `SCIPcreateExprProduct`
/// takes a coefficient). The arithmetic operators flatten as they go, so
/// `xs.iter().fold(..., |a, x| a + Expr::var(x))` builds one wide node rather than
/// a tower of binary ones — a sum over a million terms is a single
/// `SCIPcreateExprSum` call, one level deep.
///
/// Unlike [`parse_expr`](crate::ProblemOrSolving::parse_expr), an `Expr` refers to
/// variables by handle rather than by name, so it is unaffected by duplicate
/// variable names and by names containing characters the string syntax cannot
/// express (such as `>`).
///
/// Expressions are built from [`Expr::var`] and [`Expr::constant`] with the
/// arithmetic operators and the named constructors ([`Expr::pow`],
/// [`Expr::exp`], …):
///
/// ```
/// use russcip::prelude::*;
///
/// let mut model = Model::default().maximize().hide_output();
/// let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
///
/// // x^2 <= 16  =>  x <= 4
/// let e = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
/// model.add_cons_nonlinear(&e, -f64::INFINITY, 16.0, "c");
///
/// let solved = model.solve();
/// assert!((solved.obj_val() - 4.0).abs() < 1e-6);
/// ```
///
/// [`cons().expression(..)`](crate::builder::cons::ConsBuilder::expression) does
/// the building and the bound handling in one step, which is usually what you
/// want:
///
/// ```
/// use russcip::prelude::*;
///
/// let mut model = Model::default().maximize().hide_output();
/// let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
///
/// model.add(cons().expression(Expr::pow(Expr::var(&x), 2.0)).le(16.0));
///
/// let solved = model.solve();
/// assert!((solved.obj_val() - 4.0).abs() < 1e-6);
/// ```
///
/// Aggregates come from iterators:
///
/// ```
/// use russcip::prelude::*;
///
/// let mut model = Model::default().maximize().hide_output();
/// let xs: Vec<_> = (0..5)
///     .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
///     .collect();
///
/// // sum of squares <= 25
/// let e = Expr::sum(xs.iter().map(|x| Expr::pow(Expr::var(x), 2.0)));
/// let built = model.build_expr(&e).unwrap();
/// model.add_cons_nonlinear(&built, -f64::INFINITY, 25.0, "ball");
///
/// let solved = model.solve();
/// assert_eq!(solved.status(), Status::Optimal);
/// ```
#[derive(Debug, Clone)]
pub enum Expr {
    /// A model variable.
    Var(Variable),
    /// A numeric constant.
    Const(f64),
    /// `Σ cᵢ·eᵢ + constant`.
    Sum(Vec<(f64, Expr)>, f64),
    /// `coefficient · Π eᵢ`.
    Product(Vec<Expr>, f64),
    /// A subexpression raised to a constant power.
    Pow(Box<Expr>, f64),
    /// Signed power, `sign(a)*|a|^p`. Unlike [`Expr::Pow`] this is defined and
    /// odd for negative `a` at any exponent, which is why gas- and
    /// water-network models use it for direction-dependent flow terms.
    Signpower(Box<Expr>, f64),
    /// Natural exponential.
    Exp(Box<Expr>),
    /// Natural logarithm.
    Log(Box<Expr>),
    /// Sine.
    Sin(Box<Expr>),
    /// Cosine.
    Cos(Box<Expr>),
    /// Absolute value.
    Abs(Box<Expr>),
    /// Entropy, `-x*log(x)`.
    Entropy(Box<Expr>),
}

impl Expr {
    /// Refers to a model variable. Borrows, so the same variable may appear in
    /// any number of expressions.
    pub fn var(v: &Variable) -> Expr {
        Expr::Var(v.clone())
    }

    /// A numeric constant.
    pub fn constant(c: f64) -> Expr {
        Expr::Const(c)
    }

    /// Converts anything expression-like into an [`Expr`].
    ///
    /// Generic rather than a bare `.into()`, so that passing a value which is
    /// already an `Expr` does not trip clippy's `useless_conversion`.
    pub fn of(v: impl Into<Expr>) -> Expr {
        v.into()
    }

    /// `Σ eᵢ` over an iterator.
    ///
    /// ```
    /// # use russcip::prelude::*;
    /// # let mut model = Model::default().hide_output();
    /// # let xs: Vec<_> = (0..3).map(|i| model.add(var().name(&format!("x{i}")).cont(0.0..=1.0))).collect();
    /// let total = Expr::sum(xs.iter().map(Expr::var));
    /// ```
    pub fn sum<I, T>(terms: I) -> Expr
    where
        I: IntoIterator<Item = T>,
        T: Into<Expr>,
    {
        Expr::sum_weighted(terms.into_iter().map(|t| (1.0, t)))
    }

    /// `Σ cᵢ·eᵢ` over an iterator of `(coefficient, subexpression)` pairs —
    /// the usual shape of a weighted sum over indexed variables.
    ///
    /// ```
    /// # use russcip::prelude::*;
    /// # let mut model = Model::default().hide_output();
    /// # let xs: Vec<_> = (0..3).map(|i| model.add(var().name(&format!("x{i}")).cont(0.0..=1.0))).collect();
    /// let profit = [2.5, 1.0, 3.0];
    /// let e = Expr::sum_weighted(profit.iter().zip(&xs).map(|(c, x)| (*c, Expr::var(x))));
    /// ```
    pub fn sum_weighted<I, T>(terms: I) -> Expr
    where
        I: IntoIterator<Item = (f64, T)>,
        T: Into<Expr>,
    {
        let mut acc: Vec<(f64, Expr)> = Vec::new();
        let mut constant = 0.0;
        for (c, t) in terms {
            push_sum_term(&mut acc, &mut constant, c, t.into());
        }
        finish_sum(acc, constant)
    }

    /// `Π eᵢ` over an iterator.
    pub fn product<I, T>(factors: I) -> Expr
    where
        I: IntoIterator<Item = T>,
        T: Into<Expr>,
    {
        let mut acc: Vec<Expr> = Vec::new();
        let mut coef = 1.0;
        for f in factors {
            push_product_factor(&mut acc, &mut coef, f.into());
        }
        finish_product(acc, coef)
    }

    /// `a^p`, where `p` is a constant — SCIP's power expression takes a
    /// `SCIP_Real` exponent, so a variable exponent is not representable.
    pub fn pow(a: Expr, p: f64) -> Expr {
        Expr::Pow(Box::new(a), p)
    }

    /// `sign(a)*|a|^p`, with a constant exponent.
    pub fn signpower(a: Expr, p: f64) -> Expr {
        Expr::Signpower(Box::new(a), p)
    }

    /// `exp(a)`
    pub fn exp(a: Expr) -> Expr {
        Expr::Exp(Box::new(a))
    }

    /// `log(a)`
    pub fn log(a: Expr) -> Expr {
        Expr::Log(Box::new(a))
    }

    /// `sin(a)`
    pub fn sin(a: Expr) -> Expr {
        Expr::Sin(Box::new(a))
    }

    /// `cos(a)`
    pub fn cos(a: Expr) -> Expr {
        Expr::Cos(Box::new(a))
    }

    /// `abs(a)`
    pub fn abs(a: Expr) -> Expr {
        Expr::Abs(Box::new(a))
    }

    /// `entropy(a)`, i.e. `-a*log(a)`
    pub fn entropy(a: Expr) -> Expr {
        Expr::Entropy(Box::new(a))
    }
}

/// An empty aggregate is just its constant, so constant-only arithmetic folds
/// away instead of leaving a childless `Sum`/`Product` node behind. That
/// matters for constraints: a body of `x - r * r` can only move `r*r` into the
/// bound if it is a `Const`.
fn finish_sum(terms: Vec<(f64, Expr)>, constant: f64) -> Expr {
    if terms.is_empty() {
        Expr::Const(constant)
    } else {
        Expr::Sum(terms, constant)
    }
}

fn finish_product(factors: Vec<Expr>, coef: f64) -> Expr {
    if factors.is_empty() {
        Expr::Const(coef)
    } else {
        Expr::Product(factors, coef)
    }
}

/// Appends `c * t` to a sum's term list, flattening nested sums and folding
/// constants into the running constant. Flattening here is what keeps
/// `a + b + c + …` one node wide instead of `n` nodes deep.
fn push_sum_term(acc: &mut Vec<(f64, Expr)>, constant: &mut f64, c: f64, t: Expr) {
    match t {
        Expr::Const(k) => *constant += c * k,
        Expr::Sum(terms, k) => {
            *constant += c * k;
            for (ci, ti) in terms {
                push_sum_term(acc, constant, c * ci, ti);
            }
        }
        other => acc.push((c, other)),
    }
}

/// Appends a factor to a product, flattening nested products and folding
/// constants into the running coefficient.
fn push_product_factor(acc: &mut Vec<Expr>, coef: &mut f64, f: Expr) {
    match f {
        Expr::Const(k) => *coef *= k,
        Expr::Product(factors, k) => {
            *coef *= k;
            for fi in factors {
                push_product_factor(acc, coef, fi);
            }
        }
        other => acc.push(other),
    }
}

impl From<&Variable> for Expr {
    fn from(v: &Variable) -> Expr {
        Expr::Var(v.clone())
    }
}

impl From<Variable> for Expr {
    fn from(v: Variable) -> Expr {
        Expr::Var(v)
    }
}

impl From<f64> for Expr {
    fn from(c: f64) -> Expr {
        Expr::Const(c)
    }
}

/// Arithmetic on [`Expr`] — `Expr::var(&x) * 2.0 + Expr::var(&y)`.
///
/// These flatten: adding to a sum extends it rather than nesting, so folding an
/// iterator with `+` produces a single wide node.
///
/// Note that `^` is deliberately *not* implemented: Rust's `^` is `BitXor` and
/// binds looser than `+` and `*`, so `a ^ 2 + b` would mean `a ^ (2 + b)`,
/// which is not what anyone writing it would expect. Use [`Expr::pow`].
impl<R: Into<Expr>> std::ops::Add<R> for Expr {
    type Output = Expr;
    fn add(self, rhs: R) -> Expr {
        let (mut acc, mut constant) = into_sum_parts(self);
        push_sum_term(&mut acc, &mut constant, 1.0, rhs.into());
        finish_sum(acc, constant)
    }
}

impl<R: Into<Expr>> std::ops::Sub<R> for Expr {
    type Output = Expr;
    fn sub(self, rhs: R) -> Expr {
        let (mut acc, mut constant) = into_sum_parts(self);
        push_sum_term(&mut acc, &mut constant, -1.0, rhs.into());
        finish_sum(acc, constant)
    }
}

impl<R: Into<Expr>> std::ops::Mul<R> for Expr {
    type Output = Expr;
    fn mul(self, rhs: R) -> Expr {
        let (mut acc, mut coef) = into_product_parts(self);
        push_product_factor(&mut acc, &mut coef, rhs.into());
        finish_product(acc, coef)
    }
}

impl<R: Into<Expr>> std::ops::Div<R> for Expr {
    type Output = Expr;
    fn div(self, rhs: R) -> Expr {
        // SCIP has no division expression; `a / b` is `a * b^-1`.
        let (mut acc, mut coef) = into_product_parts(self);
        push_product_factor(&mut acc, &mut coef, Expr::pow(rhs.into(), -1.0));
        finish_product(acc, coef)
    }
}

impl std::ops::Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        match self {
            Expr::Const(k) => Expr::Const(-k),
            Expr::Sum(terms, k) => Expr::Sum(terms.into_iter().map(|(c, e)| (-c, e)).collect(), -k),
            Expr::Product(factors, c) => Expr::Product(factors, -c),
            other => Expr::Sum(vec![(-1.0, other)], 0.0),
        }
    }
}

macro_rules! impl_lhs_f64 {
    ($tr:ident, $method:ident) => {
        impl std::ops::$tr<Expr> for f64 {
            type Output = Expr;
            fn $method(self, rhs: Expr) -> Expr {
                std::ops::$tr::$method(Expr::Const(self), rhs)
            }
        }
    };
}

impl_lhs_f64!(Add, add);
impl_lhs_f64!(Sub, sub);
impl_lhs_f64!(Mul, mul);
impl_lhs_f64!(Div, div);

/// Decomposes an expression into sum terms so an operator can extend it in
/// place instead of nesting.
fn into_sum_parts(e: Expr) -> (Vec<(f64, Expr)>, f64) {
    match e {
        Expr::Sum(terms, k) => (terms, k),
        Expr::Const(k) => (Vec::new(), k),
        other => (vec![(1.0, other)], 0.0),
    }
}

/// Decomposes an expression into product factors, as [`into_sum_parts`] does
/// for sums.
fn into_product_parts(e: Expr) -> (Vec<Expr>, f64) {
    match e {
        Expr::Product(factors, c) => (factors, c),
        Expr::Const(k) => (Vec::new(), k),
        other => (vec![other], 1.0),
    }
}

/// Resolves a value to an [`Expr`], by type: a [`Variable`] becomes a variable
/// term and an `f64` a constant.
///
/// Useful for generic code that should accept either — a coefficient array and
/// a variable array can be walked by the same closure without annotation. The
/// blanket impl for references means it also works through however many layers
/// of borrowing an iterator binding introduces.
pub trait AsExpr {
    /// Produces the expression this value stands for.
    fn as_expr(&self) -> Expr;
}

impl AsExpr for Variable {
    fn as_expr(&self) -> Expr {
        Expr::Var(self.clone())
    }
}

impl AsExpr for f64 {
    fn as_expr(&self) -> Expr {
        Expr::Const(*self)
    }
}

impl AsExpr for Expr {
    fn as_expr(&self) -> Expr {
        self.clone()
    }
}

impl<T: AsExpr + ?Sized> AsExpr for &T {
    fn as_expr(&self) -> Expr {
        (**self).as_expr()
    }
}

impl Expr {
    /// Decomposes into linear terms plus a constant, or `None` if the
    /// expression is genuinely nonlinear.
    ///
    /// A constraint whose body is linear should become a linear constraint.
    /// SCIP would upgrade a linear `cons_nonlinear` during presolve anyway, but
    /// building the right kind up front avoids the expression tree and the
    /// upgrade round — worth doing, since an expression body is a natural way
    /// to write any constraint, not just a nonlinear one.
    pub(crate) fn as_linear(&self) -> Option<(Vec<(Variable, f64)>, f64)> {
        // A product is linear only when it is a single variable scaled by a
        // coefficient; constants have already been folded into that coefficient.
        fn scaled_var(e: &Expr) -> Option<(Variable, f64)> {
            match e {
                Expr::Var(v) => Some((v.clone(), 1.0)),
                Expr::Product(factors, c) if factors.len() == 1 => match &factors[0] {
                    Expr::Var(v) => Some((v.clone(), *c)),
                    _ => None,
                },
                _ => None,
            }
        }

        match self {
            Expr::Const(k) => Some((Vec::new(), *k)),
            Expr::Sum(terms, k) => {
                let mut out = Vec::with_capacity(terms.len());
                let mut constant = *k;
                for (c, e) in terms {
                    if let Expr::Const(kk) = e {
                        constant += c * kk;
                    } else {
                        let (v, vc) = scaled_var(e)?;
                        out.push((v, c * vc));
                    }
                }
                Some((out, constant))
            }
            other => scaled_var(other).map(|(v, c)| (vec![(v, c)], 0.0)),
        }
    }
}

/// Peels the constant off a sum, so a constraint can move it into its bounds:
/// `x^2 - 16 <= 0` becomes `x^2 <= 16`.
pub(crate) fn split_constant(ex: Expr) -> (Expr, f64) {
    match ex {
        Expr::Sum(terms, k) => (Expr::Sum(terms, 0.0), k),
        Expr::Const(k) => (Expr::Sum(Vec::new(), 0.0), k),
        other => (other, 0.0),
    }
}

impl Expr {
    /// Renders the expression as an indented tree.
    ///
    /// [`Display`](std::fmt::Display) shows the expression on one line, which is
    /// compact but hides how sums and products flattened. This shows the actual
    /// node structure:
    ///
    /// ```
    /// use russcip::prelude::*;
    ///
    /// let mut model = Model::default().hide_output();
    /// let x = model.add(var().name("x").cont(0.0..=10.0));
    /// let y = model.add(var().name("y").cont(0.0..=10.0));
    ///
    /// let e = Expr::pow(Expr::var(&x), 2.0) + 3.0 * Expr::var(&y);
    /// println!("{}", e.as_tree());
    /// // Sum
    /// // ├─ Pow(2)
    /// // │  └─ Var(x)
    /// // └─ Product ×3
    /// //    └─ Var(y)
    /// ```
    pub fn as_tree(&self) -> String {
        let mut out = String::new();
        self.write_tree(&mut out, "", true, true, None);
        out
    }

    fn write_tree(
        &self,
        out: &mut String,
        prefix: &str,
        is_root: bool,
        is_last: bool,
        coef: Option<f64>,
    ) {
        if !is_root {
            out.push_str(prefix);
            out.push_str(if is_last { "└─ " } else { "├─ " });
        }
        // A sum carries its term coefficients on the edges, not in the nodes.
        if let Some(c) = coef
            && c != 1.0
        {
            out.push_str(&format!("{c} × "));
        }
        out.push_str(&self.node_label());
        out.push('\n');

        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}   ")
        } else {
            format!("{prefix}│  ")
        };

        match self {
            Expr::Sum(terms, _) => {
                for (i, (c, e)) in terms.iter().enumerate() {
                    e.write_tree(out, &child_prefix, false, i + 1 == terms.len(), Some(*c));
                }
            }
            Expr::Product(factors, _) => {
                for (i, e) in factors.iter().enumerate() {
                    e.write_tree(out, &child_prefix, false, i + 1 == factors.len(), None);
                }
            }
            Expr::Pow(a, _)
            | Expr::Signpower(a, _)
            | Expr::Exp(a)
            | Expr::Log(a)
            | Expr::Sin(a)
            | Expr::Cos(a)
            | Expr::Abs(a)
            | Expr::Entropy(a) => a.write_tree(out, &child_prefix, false, true, None),
            Expr::Var(_) | Expr::Const(_) => {}
        }
    }

    fn node_label(&self) -> String {
        match self {
            Expr::Var(v) => format!("Var({})", v.name()),
            Expr::Const(c) => format!("Const({c})"),
            Expr::Sum(_, k) if *k != 0.0 => format!("Sum (+{k})"),
            Expr::Sum(..) => "Sum".to_string(),
            Expr::Product(_, c) if *c != 1.0 => format!("Product ×{c}"),
            Expr::Product(..) => "Product".to_string(),
            Expr::Pow(_, p) => format!("Pow({p})"),
            Expr::Signpower(_, p) => format!("Signpower({p})"),
            Expr::Exp(_) => "Exp".to_string(),
            Expr::Log(_) => "Log".to_string(),
            Expr::Sin(_) => "Sin".to_string(),
            Expr::Cos(_) => "Cos".to_string(),
            Expr::Abs(_) => "Abs".to_string(),
            Expr::Entropy(_) => "Entropy".to_string(),
        }
    }
}

/// Parenthesised rendering, so the grouping is visible.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(v) => write!(f, "<{}>", v.name()),
            Expr::Const(c) => write!(f, "{c}"),

            Expr::Sum(terms, constant) => {
                if terms.is_empty() {
                    return write!(f, "{constant}");
                }
                // A lone unweighted term needs no wrapping.
                if terms.len() == 1 && terms[0].0 == 1.0 && *constant == 0.0 {
                    return write!(f, "{}", terms[0].1);
                }
                write!(f, "(")?;
                for (i, (c, e)) in terms.iter().enumerate() {
                    if i == 0 {
                        if *c == -1.0 {
                            write!(f, "-{e}")?;
                        } else if *c == 1.0 {
                            write!(f, "{e}")?;
                        } else {
                            write!(f, "{c}*{e}")?;
                        }
                    } else if *c < 0.0 {
                        if *c == -1.0 {
                            write!(f, " - {e}")?;
                        } else {
                            write!(f, " - {}*{e}", -c)?;
                        }
                    } else if *c == 1.0 {
                        write!(f, " + {e}")?;
                    } else {
                        write!(f, " + {c}*{e}")?;
                    }
                }
                if *constant != 0.0 {
                    if *constant < 0.0 {
                        write!(f, " - {}", -constant)?;
                    } else {
                        write!(f, " + {constant}")?;
                    }
                }
                write!(f, ")")
            }

            Expr::Product(factors, coef) => {
                if factors.is_empty() {
                    return write!(f, "{coef}");
                }
                if factors.len() == 1 && *coef == 1.0 {
                    return write!(f, "{}", factors[0]);
                }
                write!(f, "(")?;
                if *coef != 1.0 {
                    write!(f, "{coef} * ")?;
                }
                for (i, e) in factors.iter().enumerate() {
                    // `a * b^-1` reads better as `a / b`.
                    if let Expr::Pow(base, p) = e
                        && *p == -1.0
                        && i > 0
                    {
                        write!(f, " / {base}")?;
                        continue;
                    }
                    if i > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{e}")?;
                }
                write!(f, ")")
            }

            Expr::Pow(a, p) => write!(f, "({a}^{p})"),
            Expr::Signpower(a, p) => write!(f, "signpower({a}, {p})"),
            Expr::Exp(a) => write!(f, "exp({a})"),
            Expr::Log(a) => write!(f, "log({a})"),
            Expr::Sin(a) => write!(f, "sin({a})"),
            Expr::Cos(a) => write!(f, "cos({a})"),
            Expr::Abs(a) => write!(f, "abs({a})"),
            Expr::Entropy(a) => write!(f, "entropy({a})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn solves_quadratic() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

        // x^2 <= 16  =>  x <= 4
        let e = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, 16.0, "c");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!((solved.obj_val() - 4.0).abs() < 1e-6);
    }

    /// The one-line rendering, and with it the grouping the operators produced.
    #[test]
    fn display_shows_the_grouping() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(0.0..=10.0));
        let y = model.add(var().name("y").cont(0.0..=10.0));

        let e = Expr::pow(Expr::var(&x), 2.0) + 3.0 * Expr::var(&y);
        assert_eq!(e.to_string(), "((<x>^2) + (3 * <y>))");

        assert_eq!((-Expr::pow(Expr::var(&x), 2.0)).to_string(), "(-(<x>^2))");
        assert_eq!(
            (3.0 * Expr::pow(Expr::var(&x), 2.0)).to_string(),
            "(3 * (<x>^2))"
        );
        assert_eq!(
            Expr::pow(Expr::var(&x) + Expr::var(&y), 2.0).to_string(),
            "((<x> + <y>)^2)"
        );
    }

    /// Sums and products flatten as the operators go, so a chain is one wide
    /// node rather than a tower of binary ones.
    #[test]
    fn operator_chains_flatten() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(1.0..=10.0));
        let y = model.add(var().name("y").cont(1.0..=10.0));

        let diff = Expr::var(&x) - Expr::var(&y) - Expr::var(&x);
        assert_eq!(diff.to_string(), "(<x> - <y> - <x>)");
        assert!(matches!(&diff, Expr::Sum(terms, _) if terms.len() == 3));

        // Division is `a * b^-1`, so a division chain is one product.
        let quot = Expr::var(&x) / Expr::var(&y) / 2.0;
        assert_eq!(quot.to_string(), "(<x> / <y> / 2)");
        assert!(matches!(&quot, Expr::Product(factors, _) if factors.len() == 3));
    }

    /// The string API resolves `<x>` by name; the builder uses handles, so
    /// duplicate names are unambiguous.
    #[test]
    fn duplicate_variable_names_are_unambiguous() {
        let mut model = Model::default().maximize().hide_output();
        let x1 = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        let _x2 = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

        // Constrain *specifically* x1: x1^2 <= 4  =>  x1 <= 2. x2 stays at 10.
        let e = model.build_expr(&Expr::pow(Expr::var(&x1), 2.0)).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, 4.0, "c");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 12.0).abs() < 1e-6,
            "expected 2 + 10 = 12, got {}",
            solved.obj_val()
        );
    }

    /// A name containing `>` cannot be written in SCIP's `<name>` syntax at all.
    #[test]
    fn names_the_string_syntax_cannot_express() {
        let mut model = Model::default().maximize().hide_output();
        let v = model.add(var().name("a>b").obj(1.).cont(0.0..=10.0));

        assert!(model.parse_expr("<a>b>^2").is_err(), "string route works?");

        let e = model.build_expr(&Expr::pow(Expr::var(&v), 2.0)).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, 9.0, "c");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!((solved.obj_val() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn division_and_functions() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(1.0..=10.0));

        // log(x) + 1/x <= log(4) + 0.25  =>  x <= 4 on this range
        let rhs = 4f64.ln() + 0.25;
        let body = Expr::log(Expr::var(&x)) + 1.0 / Expr::var(&x);
        let e = model.build_expr(&body).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, rhs, "c");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }

    /// Rosenbrock, minimised through an auxiliary variable since SCIP has no
    /// nonlinear objective. Optimum is 0 at (1, 1).
    #[test]
    fn rosenbrock() {
        let mut model = Model::default().minimize().hide_output();
        let x = model.add(var().name("x").cont(-2.0..=2.0));
        let y = model.add(var().name("y").cont(-2.0..=2.0));
        let t = model.add(var().name("t").obj(1.).cont(0.0..=1e6));

        // (1 - x)^2 + 100*(y - x^2)^2 - t <= 0
        let body = Expr::pow(1.0 - Expr::var(&x), 2.0)
            + 100.0 * Expr::pow(Expr::var(&y) - Expr::pow(Expr::var(&x), 2.0), 2.0)
            - Expr::var(&t);
        let e = model.build_expr(&body).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, 0.0, "rosenbrock");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(solved.obj_val() < 1e-4, "got {}", solved.obj_val());

        let sol = solved.best_sol().unwrap();
        assert!((sol.val(&x) - 1.0).abs() < 1e-2, "x = {}", sol.val(&x));
        assert!((sol.val(&y) - 1.0).abs() < 1e-2, "y = {}", sol.val(&y));
    }

    /// Builds and drops many expressions to shake out reference-count errors:
    /// a missing release leaks, an extra one is a double free.
    #[test]
    fn refcounting_stress() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(1.0..=10.0));
        let y = model.add(var().name("y").cont(1.0..=10.0));

        for _ in 0..2000 {
            let e = Expr::exp(Expr::var(&x))
                + Expr::log(Expr::var(&y))
                    / (Expr::pow(Expr::var(&x), 2.0) - Expr::abs(Expr::var(&y)))
                    * Expr::sin(Expr::var(&x) + Expr::var(&y))
                - Expr::pow(Expr::cos(Expr::var(&x)), 3.0);
            let built = model.build_expr(&e).unwrap();
            drop(built);
        }

        // still usable afterwards
        let e = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
        model.add_cons_nonlinear(&e, -f64::INFINITY, 16.0, "c");
        let solved = model.solve();
        assert!((solved.obj_val() - 4.0).abs() < 1e-4);
    }

    /// `signpower(x, n) = sign(x)|x|^n`, so it is odd and takes negative values.
    /// Plain `x^3` agrees with it here; the test pins the sign behaviour.
    #[test]
    fn signpower_is_odd() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(-5.0..=5.0));

        // signpower(x,3) == -8  =>  x == -2. With |x|^3 this is infeasible.
        let e = model
            .build_expr(&Expr::signpower(Expr::var(&x), 3.0))
            .unwrap();
        model.add_cons_nonlinear(&e, -8.0, -8.0, "sp");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        let sol = solved.best_sol().unwrap();
        assert!((sol.val(&x) + 2.0).abs() < 1e-4, "x = {}", sol.val(&x));
    }

    /// The case a plain power cannot express: a fractional exponent applied to
    /// a negative argument. `x^1.5` is undefined there.
    #[test]
    fn signpower_fractional_exponent() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(-4.0..=4.0));

        // signpower(x, 1.5) == -8  =>  |x|^1.5 = 8  =>  x = -4
        let e = model
            .build_expr(&Expr::signpower(Expr::var(&x), 1.5))
            .unwrap();
        model.add_cons_nonlinear(&e, -8.0, -8.0, "sp");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        let sol = solved.best_sol().unwrap();
        assert!((sol.val(&x) + 4.0).abs() < 1e-3, "x = {}", sol.val(&x));
    }

    #[test]
    fn signpower_renders_and_composes() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(-5.0..=5.0));
        let y = model.add(var().name("y").cont(-5.0..=5.0));

        assert_eq!(
            Expr::signpower(Expr::var(&x) + Expr::var(&y), 2.0).to_string(),
            "signpower((<x> + <y>), 2)"
        );
        assert_eq!(
            (2.0 * Expr::signpower(Expr::var(&x), -1.5) + Expr::var(&y)).to_string(),
            "((2 * signpower(<x>, -1.5)) + <y>)"
        );
    }

    /// An aggregate is an ordinary [`Expr`], so it composes with the operators.
    #[test]
    fn aggregates_compose_with_operators() {
        let mut model = Model::default().maximize().hide_output();
        let xs: Vec<_> = (0..3)
            .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
            .collect();
        let y = model.add(var().name("y").obj(1.).cont(0.0..=10.0));

        // `&Variable: Into<Expr>`, so no `.map(Expr::var)` is needed.
        let total = Expr::sum(xs.iter());
        assert_eq!(total.to_string(), "(<x0> + <x1> + <x2>)");

        // Extending the aggregate widens it rather than nesting it.
        let e = total + Expr::pow(Expr::var(&y), 2.0);
        assert_eq!(e.to_string(), "(<x0> + <x1> + <x2> + (<y>^2))");
        assert!(matches!(&e, Expr::Sum(terms, _) if terms.len() == 4));

        let built = model.build_expr(&e).unwrap();
        model.add_cons_nonlinear(&built, -f64::INFINITY, 4.0, "c");
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
    }

    /// Aggregates over iterators — the common modelling case.
    #[test]
    fn sum_and_product_over_iterators() {
        let mut model = Model::default().hide_output();
        let xs: Vec<_> = (0..3)
            .map(|i| model.add(var().name(&format!("x{i}")).cont(0.0..=10.0)))
            .collect();

        assert_eq!(
            Expr::sum(xs.iter().map(Expr::var)).to_string(),
            "(<x0> + <x1> + <x2>)"
        );

        let profit = [2.5, 1.0, -3.0];
        assert_eq!(
            Expr::sum_weighted(profit.iter().zip(&xs).map(|(c, x)| (*c, Expr::var(x)))).to_string(),
            "(2.5*<x0> + <x1> - 3*<x2>)"
        );

        assert_eq!(
            Expr::product(xs.iter().map(Expr::var)).to_string(),
            "(<x0> * <x1> * <x2>)"
        );

        // Empty aggregates are the identity elements.
        assert_eq!(Expr::sum(Vec::<Expr>::new()).to_string(), "0");
        assert_eq!(Expr::product(Vec::<Expr>::new()).to_string(), "1");
    }

    /// A weighted sum of squares, solved end to end.
    #[test]
    fn sum_of_squares_constraint() {
        let mut model = Model::default().maximize().hide_output();
        let xs: Vec<_> = (0..4)
            .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
            .collect();

        // Σ xᵢ² <= 4, maximise Σ xᵢ  =>  all equal at 1, objective 4.
        let e = Expr::sum(xs.iter().map(|x| Expr::pow(Expr::var(x), 2.0)));
        let built = model.build_expr(&e).unwrap();
        model.add_cons_nonlinear(&built, -f64::INFINITY, 4.0, "ball");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }

    /// Regression: a wide sum must stay one node deep. A binary tree here
    /// overflowed the stack during lowering at this size.
    #[test]
    fn wide_sums_do_not_recurse() {
        let mut model = Model::default().maximize().hide_output();
        let xs: Vec<_> = (0..20000)
            .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=1.0)))
            .collect();

        // Both spellings must stay flat: the iterator constructor...
        let e = Expr::sum(xs.iter().map(Expr::var));
        assert!(matches!(&e, Expr::Sum(terms, _) if terms.len() == 20000));
        assert!(model.build_expr(&e).is_ok());

        // ...and folding with the `+` operator.
        let folded = xs
            .iter()
            .skip(1)
            .fold(Expr::var(&xs[0]), |acc, v| acc + Expr::var(v));
        assert!(matches!(&folded, Expr::Sum(terms, _) if terms.len() == 20000));
        assert!(model.build_expr(&folded).is_ok());
    }

    // ---- cons().expression(..) ----

    #[test]
    fn cons_expression_simple() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        model.add(cons().expression(Expr::pow(Expr::var(&x), 2.0)).le(16.0));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-6,
            "got {}",
            solved.obj_val()
        );
    }

    /// The two-sided form, `lhs <= body <= rhs`.
    #[test]
    fn cons_expression_two_sided() {
        let mut model = Model::default().minimize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        let y = model.add(var().name("y").obj(1.).cont(0.0..=10.0));
        model.add(
            cons()
                .expression(Expr::var(&x) + Expr::var(&y))
                .bounds(3.0, 5.0)
                .name("band"),
        );
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 3.0).abs() < 1e-6,
            "got {}",
            solved.obj_val()
        );
    }

    #[test]
    fn cons_expression_equality() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.1..=10.0));
        let y = model.add(var().name("y").cont(0.1..=10.0));
        model.add(cons().expression(Expr::var(&x) * Expr::var(&y)).eq(1.0));
        model.add(cons().expression(Expr::var(&y)).ge(0.25));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }

    /// Variables on both sides of the comparison are written as one body:
    /// `x^2 <= y` is `x^2 - y <= 0`.
    #[test]
    fn cons_expression_variables_on_both_sides() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        let y = model.add(var().name("y").cont(0.0..=4.0));
        model.add(
            cons()
                .expression(Expr::pow(Expr::var(&x), 2.0) - Expr::var(&y))
                .le(0.0),
        );
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 2.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }

    /// A linear body still routes through the builder correctly.
    #[test]
    fn cons_expression_linear_body() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        let y = model.add(var().name("y").obj(1.).cont(0.0..=10.0));
        model.add(
            cons()
                .expression(Expr::var(&x) + 2.0 * Expr::var(&y))
                .le(10.0),
        );
        model.add(cons().expression(Expr::var(&x)).le(4.0));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 7.0).abs() < 1e-6,
            "got {}",
            solved.obj_val()
        );
    }

    // ---- aggregates as constraint bodies ----

    #[test]
    fn sum_over_collection_as_body() {
        let mut model = Model::default().maximize().hide_output();
        let xs: Vec<_> = (0..4)
            .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
            .collect();

        let squares = Expr::sum(xs.iter().map(|x| Expr::pow(Expr::var(x), 2.0)));
        assert_eq!(
            squares.to_string(),
            "((<x0>^2) + (<x1>^2) + (<x2>^2) + (<x3>^2))"
        );

        model.add(cons().expression(squares).le(4.0));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }

    /// A weighted sum over indices. The coefficients ride on the sum's edges
    /// rather than becoming `Product` nodes, which is why they render as
    /// `2*<x1>` and not `(2 * <x1>)`.
    #[test]
    fn weighted_sum_over_indices_as_body() {
        let mut model = Model::default().maximize().hide_output();
        let n = 3;
        let xs: Vec<_> = (0..n)
            .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
            .collect();
        let c = [1.0, 2.0, 4.0];

        let weighted = Expr::sum_weighted((0..n).map(|i| (c[i], Expr::var(&xs[i]))));
        assert_eq!(weighted.to_string(), "(<x0> + 2*<x1> + 4*<x2>)");

        model.add(cons().expression(weighted).le(8.0));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        // best is to load x0 and x1 to their bound then x2 with what is left
        assert!(solved.obj_val() > 0.0);
    }

    #[test]
    fn product_over_collection_as_body() {
        let mut model = Model::default().hide_output();
        let xs: Vec<_> = (0..3)
            .map(|i| model.add(var().name(&format!("x{i}")).cont(0.5..=2.0)))
            .collect();

        let prod = Expr::product(xs.iter().map(Expr::var));
        assert_eq!(prod.to_string(), "(<x0> * <x1> * <x2>)");

        model.add(cons().expression(prod).eq(1.0));
        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
    }

    /// [`AsExpr`] resolves a value by type, so a coefficient array and a
    /// variable array can be walked by one closure.
    #[test]
    fn as_expr_resolves_by_type() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(0.0..=10.0));
        let a = 2.5f64;

        assert_eq!(a.as_expr().to_string(), "2.5");
        assert_eq!(x.as_expr().to_string(), "<x>");
        assert_eq!((a.as_expr() * x.as_expr()).to_string(), "(2.5 * <x>)");

        // and through a reference, as an iterator binding gives it
        assert_eq!((&&a).as_expr().to_string(), "2.5");
    }

    /// A linear body is recognised as such, so the builder makes a genuine
    /// linear constraint rather than one presolve has to upgrade.
    #[test]
    fn linear_bodies_are_detected() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(0.0..=10.0));
        let y = model.add(var().name("y").cont(0.0..=10.0));

        let body = 2.0 * Expr::var(&x) + 3.0 * Expr::var(&y) - 4.0;
        let (terms, k) = body.as_linear().expect("linear");
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].1, 2.0);
        assert_eq!(terms[1].1, 3.0);
        assert_eq!(k, -4.0);

        assert!(Expr::var(&x).as_linear().is_some());
        assert!((-Expr::var(&x)).as_linear().is_some());
        assert!(Expr::sum([&x, &y]).as_linear().is_some());

        assert!(Expr::pow(Expr::var(&x), 2.0).as_linear().is_none());
        assert!((Expr::var(&x) * Expr::var(&y)).as_linear().is_none());
        assert!(Expr::exp(Expr::var(&x)).as_linear().is_none());
        assert!((Expr::var(&x) / Expr::var(&y)).as_linear().is_none());
    }

    /// The tree view shows structure the one-line form hides: the flattening of
    /// a subtraction chain, and division lowered to a negative power.
    #[test]
    fn as_tree_shows_the_real_structure() {
        let mut model = Model::default().hide_output();
        let x = model.add(var().name("x").cont(1.0..=10.0));
        let y = model.add(var().name("y").cont(1.0..=10.0));

        assert_eq!(
            (Expr::pow(Expr::var(&x), 2.0) + 3.0 * Expr::var(&y)).as_tree(),
            "Sum\n\
             ├─ Pow(2)\n\
             │  └─ Var(x)\n\
             └─ Product ×3\n\
             \u{20}  └─ Var(y)\n"
        );

        // one flat sum with -1 coefficients, not nested subtractions
        assert_eq!(
            (Expr::var(&x) - Expr::var(&y) - Expr::var(&x)).as_tree(),
            "Sum\n├─ Var(x)\n├─ -1 × Var(y)\n└─ -1 × Var(x)\n"
        );

        // the `+1` is the sum's constant; `/ y` is `y^-1` inside a product
        assert_eq!(
            (Expr::exp(Expr::var(&x)) / Expr::var(&y) + 1.0).as_tree(),
            "Sum (+1)\n\
             └─ Product\n\
             \u{20}  ├─ Exp\n\
             \u{20}  │  └─ Var(x)\n\
             \u{20}  └─ Pow(-1)\n\
             \u{20}     └─ Var(y)\n"
        );
    }

    /// Constant-only arithmetic folds to a constant, so it reaches the bound
    /// instead of becoming a childless product node.
    #[test]
    fn constant_arithmetic_folds() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
        let r = 3.0f64;

        assert_eq!((Expr::constant(r) * r).to_string(), "9");
        assert_eq!((2.0 * Expr::constant(r) + 1.0).to_string(), "7");

        // x^2 - r*r <= 0  =>  the -9 moves into the bound, so x <= 3
        let body = Expr::pow(Expr::var(&x), 2.0) - Expr::constant(r) * r;
        model.add(cons().expression(body).le(0.0));
        let solved = model.solve();
        assert!(
            (solved.obj_val() - 3.0).abs() < 1e-6,
            "got {}",
            solved.obj_val()
        );
    }

    /// The operator traits, including the ones with `f64` on the left.
    #[test]
    fn operators_build_expressions() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

        let e = 2.0 * Expr::var(&x) + 1.0;
        assert_eq!(e.to_string(), "((2 * <x>) + 1)");
        assert_eq!((1.0 - Expr::var(&x)).to_string(), "(-<x> + 1)");
        assert_eq!((1.0 / Expr::var(&x)).to_string(), "(<x>^-1)");

        let built = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
        model.add_cons_nonlinear(&built, -f64::INFINITY, 16.0, "c");
        let solved = model.solve();
        assert!((solved.obj_val() - 4.0).abs() < 1e-6);
    }

    /// Nothing is moved, so a variable can appear in many expressions.
    #[test]
    fn variables_are_reusable() {
        let mut model = Model::default().maximize().hide_output();
        let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));

        let a = model.build_expr(&Expr::pow(Expr::var(&x), 2.0)).unwrap();
        let b = model
            .build_expr(&(Expr::pow(Expr::var(&x), 2.0) + Expr::var(&x)))
            .unwrap();
        model.add_cons_nonlinear(&a, -f64::INFINITY, 25.0, "c1");
        model.add_cons_nonlinear(&b, -f64::INFINITY, 20.0, "c2");

        let solved = model.solve();
        assert_eq!(solved.status(), Status::Optimal);
        assert!(
            (solved.obj_val() - 4.0).abs() < 1e-4,
            "got {}",
            solved.obj_val()
        );
    }
}
