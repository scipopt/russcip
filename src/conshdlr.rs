use crate::scip::ScipPtr;
use crate::{ffi, Model, Solution, Solving, Variable};
use std::rc::Rc;

/// A trait for implementing custom constraint handlers.
pub trait Conshdlr {
    /// Check if the (primal) solution satisfies the constraint.
    fn check(&mut self, model: Model<Solving>, conshdlr: SCIPConshdlr, solution: Solution) -> bool;

    /// Enforce the constraint for the current sub-problem's (LP) solution.
    fn enforce(&mut self, model: Model<Solving>, conshdlr: SCIPConshdlr) -> ConshdlrResult;

    /// Prevents rounding of variables in constraints.
    fn locks(&mut self, model: Model<Solving>, conshdlr: SCIPConshdlr, var: &Variable) -> LockDirection {
        LockDirection::Both
    }
}

/// The result of enforcing a constraint handler.
pub enum ConshdlrResult {
    /// States that the problem is feasible.
    Feasible,
    /// States that the problem is infeasible.
    CutOff,
    /// Added another constraint that resolves the infeasibility.
    ConsAdded,
    /// Reduced the domain of a variable.
    ReducedDom,
    /// Added a cutting plane that separates the lp solution.
    Separated,
    /// Request to resolve the LP.
    SolveLP,
    /// Created a branching.
    Branched,
}

/// The direction in which a variable should be locked (from being rounded).
pub enum LockDirection {
    /// Rounding down the variable can cause infeasibility.
    Lower,
    /// Rounding up the variable can cause infeasibility.
    Upper,
    /// Both rounding directions can cause infeasibility.
    Both,
}

/// Wrapper for the internal SCIP constraint handler.
pub struct SCIPConshdlr {
    pub(crate) raw: *mut ffi::SCIP_CONSHDLR,
    pub(crate) scip: Rc<ScipPtr>,
}
