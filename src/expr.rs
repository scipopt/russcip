use crate::scip::ScipPtr;
use crate::{ffi, scip_call_panic};
use std::rc::Rc;

/// A SCIP expression (`SCIP_EXPR`), used to build nonlinear constraints.
///
/// An `Expr` is reference-counted by SCIP. This wrapper owns one reference and
/// releases it on `Drop`, so the underlying expression is freed automatically
/// once no constraint (or parent expression) still holds it. The retained
/// [`Rc<ScipPtr>`] keeps the owning model alive for at least as long as the
/// expression.
#[derive(Debug)]
pub struct Expr {
    /// A pointer to the underlying `SCIP_EXPR` C struct.
    pub(crate) raw: *mut ffi::SCIP_EXPR,
    /// A reference to the SCIP instance that owns this expression.
    pub(crate) scip: Rc<ScipPtr>,
}

impl Expr {
    /// Returns a pointer to the underlying `SCIP_EXPR` C struct.
    pub fn inner(&self) -> *mut ffi::SCIP_EXPR {
        self.raw
    }
}

impl Drop for Expr {
    fn drop(&mut self) {
        // Decrement the expression's usage count, SCIP frees it once it reaches 0.
        scip_call_panic!(ffi::SCIPreleaseExpr(self.scip.raw, &mut self.raw));
    }
}
