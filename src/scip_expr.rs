use crate::scip::ScipPtr;
use crate::{ffi, scip_call_panic};
use std::rc::Rc;

/// A SCIP expression (`SCIP_EXPR`), used to build nonlinear constraints.
///
/// An `ScipExpr` is reference-counted by SCIP. This wrapper owns one reference and
/// releases it on `Drop`, so the underlying expression is freed automatically
/// once no constraint (or parent expression) still holds it. The retained
/// [`Rc<ScipPtr>`] keeps the owning model alive for at least as long as the
/// expression.
///
/// # Creating an expression
///
/// Expressions are built by parsing a string with
/// [`parse_expr`](crate::ProblemOrSolving::parse_expr). Reference a variable by
/// its name wrapped in angle brackets (`<name>`); the name is resolved against
/// the variables already added to the model, so add the variables first. SCIP's
/// syntax supports the usual operators (`+ - * / ^`) and functions such as
/// `exp`, `log`, `sqrt`, `sin`, `cos`, and `abs`. The string parser accepts
/// `sqrt` (lowering it to `x^0.5`); `Expr` has no `Sqrt` variant, so write
/// `Expr::pow(x, 0.5)` when building by hand.
///
/// ```
/// use russcip::prelude::*;
///
/// let mut model = Model::default().maximize().hide_output();
/// model.add(var().name("x").obj(1.).cont(0.0..=10.0));
///
/// // Parse `x^2` into an expression, then bound it: x^2 <= 16  =>  x <= 4.
/// let expr = model.parse_expr("<x>^2").unwrap();
/// model.add_cons_nonlinear(&expr, -f64::INFINITY, 16.0, "c");
///
/// let solved = model.solve();
/// assert!((solved.obj_val() - 4.0).abs() < 1e-6);
/// ```
///
/// The same `expr` may be reused in several constraints: `add_cons_nonlinear`
/// takes it by reference and the constraint keeps its own copy.
///
/// # See also
///
/// [`build_expr`](crate::ProblemOrSolving::build_expr) constructs an `ScipExpr`
/// from an [`Expr`](crate::Expr) tree instead of from a string. That route
/// refers to variables by handle, so it is unaffected by duplicate variable
/// names and by names containing characters this string syntax cannot express
/// (such as `>`).
#[derive(Debug)]
pub struct ScipExpr {
    /// A pointer to the underlying `SCIP_EXPR` C struct.
    pub(crate) raw: *mut ffi::SCIP_EXPR,
    /// A reference to the SCIP instance that owns this expression.
    pub(crate) scip: Rc<ScipPtr>,
}

impl ScipExpr {
    /// Returns a pointer to the underlying `SCIP_EXPR` C struct.
    pub fn inner(&self) -> *mut ffi::SCIP_EXPR {
        self.raw
    }
}

impl Drop for ScipExpr {
    fn drop(&mut self) {
        // Decrement the expression's usage count, SCIP frees it once it reaches 0.
        scip_call_panic!(ffi::SCIPreleaseExpr(self.scip.raw, &mut self.raw));
    }
}
