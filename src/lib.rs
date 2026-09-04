//! # russcip
//! Safe Rust interface for [SCIP](https://scipopt.org/) optimization suite.
//!
//! For usage, please refer to the [README](https://github.com/scipopt/russcip).
//!
//! # Example
//! ```rust
//! use russcip::prelude::*;
//!
//! let mut model = Model::default().minimize();
//! let x = model.add(var().bin().obj(1.0));
//! let y = model.add(var().bin().obj(2.0));
//! model.add(cons().coef(&x, 1.0).coef(&y, 1.0).eq(1.0));
//!
//! let solved = model.solve();
//! assert_eq!(solved.status(), Status::Optimal);
//! assert_eq!(solved.obj_val(), 1.0);
//! ```
//!
//! # Nonlinear constraints
//!
//! [`Expr`] is an expression tree built from variables and constants. It is
//! plain Rust data — constructing it touches SCIP not at all — and
//! [`ConsBuilder::expression`](crate::builder::cons::ConsBuilder::expression)
//! turns one into a constraint:
//!
//! ```rust
//! use russcip::prelude::*;
//!
//! let mut model = Model::default().maximize().hide_output();
//! let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
//! let y = model.add(var().name("y").cont(0.0..=10.0));
//!
//! // x² + y² <= 16
//! let circle = Expr::pow(Expr::var(&x), 2.0) + Expr::pow(Expr::var(&y), 2.0);
//! model.add(cons().expression(circle).le(16.0));
//!
//! // 1 <= x + y <= 5
//! model.add(cons().expression(Expr::var(&x) + Expr::var(&y)).bounds(1.0, 5.0));
//!
//! let solved = model.solve();
//! assert_eq!(solved.status(), Status::Optimal);
//! ```
//!
//! The [`cons!`] and [`expr!`] macros write the same thing as mathematical
//! syntax, with `^` binding tighter than `*` (unlike Rust's `^`, which is
//! bitwise xor). A comprehension makes an aggregate over an iterator:
//!
//! ```rust
//! use russcip::prelude::*;
//!
//! let mut model = Model::default().maximize().hide_output();
//! let x = model.add(var().name("x").obj(1.).cont(0.0..=10.0));
//! let y = model.add(var().name("y").cont(0.0..=10.0));
//! let n = 4;
//! let xs: Vec<_> = (0..n)
//!     .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
//!     .collect();
//!
//! // x² + y² <= 16
//! model.add(cons!(x ^ 2 + y ^ 2 <= 16));
//! // 1 <= x + y <= 5
//! model.add(cons!(1 <= x + y <= 5));
//! // Σ xᵢ² <= 4
//! model.add(cons!(sum(i in 0..n, xs[i] ^ 2) <= 4));
//!
//! let solved = model.solve();
//! assert_eq!(solved.status(), Status::Optimal);
//! ```
//!
//! Aggregates come from iterators, so a coefficient array and a variable array
//! pair up with `zip`:
//!
//! ```rust
//! use russcip::prelude::*;
//!
//! let mut model = Model::default().maximize().hide_output();
//! let n = 4;
//! let xs: Vec<_> = (0..n)
//!     .map(|i| model.add(var().name(&format!("x{i}")).obj(1.).cont(0.0..=10.0)))
//!     .collect();
//! let c = [1.0, 2.0, 3.0, 0.5];
//!
//! // Σ cᵢ·xᵢ <= 10
//! let weighted = Expr::sum_weighted(c.iter().zip(&xs).map(|(c, x)| (*c, Expr::var(x))));
//! model.add(cons().expression(weighted).le(10.0));
//!
//! // Σ xᵢ² <= 4
//! let squares = Expr::sum(xs.iter().map(|x| Expr::pow(Expr::var(x), 2.0)));
//! model.add(cons().expression(squares).le(4.0));
//!
//! let solved = model.solve();
//! assert_eq!(solved.status(), Status::Optimal);
//! ```
//!
//! See [`expr`](mod@crate::expr) for the full API and
//! [`parse_expr`](ProblemOrSolving::parse_expr) for SCIP's string syntax.

#![deny(missing_docs)]
#![allow(clippy::macro_metavars_in_unsafe)]
extern crate core;

/// Re-exports the `scip_sys` crate, which provides low-level bindings to the SCIP library.
pub use scip_sys as ffi;

// Lets the `expr!` macro's `::russcip::…` paths resolve inside this crate too.
extern crate self as russcip;

/// Contains the `BranchRule` trait used to define custom branching rules.
pub mod branchrule;
pub use branchrule::*;

/// Contains the `Constraint` struct, which represents a constraint in an optimization problem.
pub mod constraint;
pub use constraint::*;

/// Contains the `Expr` enum, a model-independent description of a nonlinear
/// expression. This is the type you build; [`ScipExpr`] is what SCIP makes of it.
pub mod expr;
pub use expr::*;

/// Contains the `ScipExpr` struct, a handle on a `SCIP_EXPR` built by SCIP.
pub mod scip_expr;
pub use scip_expr::*;

/// Builds an [`Expr`] expression tree from mathematical syntax, with `^` binding
/// tighter than `*` (unlike Rust's `^`, which is `BitXor`).
pub use russcip_macros::expr;

/// Builds a constraint from a comparison (`<=`, `>=`, `=`), ready for
/// [`Model::add`].
pub use russcip_macros::cons;

/// The main module, it contains the `Model` struct, which represents an optimization problem.
pub mod model;
pub use model::*;

/// Contains the `Pricer` trait used to define custom variable pricing strategies.
pub mod pricer;
pub use pricer::*;

/// Contains the `Retcode` enum, which represents the return codes of SCIP functions.
pub mod retcode;
pub use retcode::*;

/// Contains the `Solution` struct, which represents a solution to an optimization problem.
pub mod solution;
pub use solution::*;

/// Contains the `Status` enum, which represents the status of an optimization problem.
pub mod status;
pub use status::*;

/// Contains the `Variable` struct, which represents a variable in an optimization problem.
pub mod variable;
pub use variable::*;

/// Contains the `Node` struct, which represents a node in the branch-and-bound tree.
pub mod node;
pub use node::*;

/// Contains the `EventHdlr` trait used to define custom event handlers.
pub mod eventhdlr;
pub use eventhdlr::*;

/// Contains the `Heur` trait used to define custom primal heuristics.
pub mod heuristic;
pub use heuristic::*;

/// Contains the `Separator` trait used to define custom separation routines.
pub mod separator;
pub use separator::*;

/// Contains the `NodeSel` trait used to define custom node selectors.
pub mod nodesel;
pub use nodesel::*;

/// Contains all the traits and structs that are re-exported by default.
pub mod prelude;

mod scip;

/// Contains the `Col` struct, which represents a column in an LP relaxation.
pub mod col;
pub use col::*;

mod param;
/// Contains the `Row` struct, which represents a row in an LP relaxation.
pub mod row;

/// Contains methods for creating scip objects in an ergonomic way.
pub mod builder;

/// Contains the `Conshdlr` trait used to define custom constraint handlers.
pub mod conshdlr;
mod diving;
mod probing;

pub use conshdlr::*;
pub use diving::*;

pub use row::*;

/// A macro for calling a `SCIP` function and returning an error if the return code is not `SCIP_OKAY`.
#[macro_export]
macro_rules! scip_call {
    ($res:expr) => {
        let res = unsafe { $res };
        let retcode = $crate::retcode::Retcode::from(res);
        if retcode != $crate::retcode::Retcode::Okay {
            return Err(retcode);
        }
    };
}

/// A macro for calling a `SCIP` function and panicking if the return code is not `SCIP_OKAY`.
#[macro_export]
macro_rules! scip_call_panic {
    ($res:expr) => {
        let res = unsafe { $res };
        let retcode = $crate::retcode::Retcode::from(res);
        if retcode != $crate::retcode::Retcode::Okay {
            panic!("SCIP call failed with retcode {:?}", retcode);
        }
    };
}

/// A macro for calling a `SCIP` function and panicking with a custom message if the return code is not `SCIP_OKAY`.
#[macro_export]
macro_rules! scip_call_expect {
    ($res:expr, $msg:expr) => {
        let res = unsafe { $res };
        let retcode = $crate::retcode::Retcode::from(res);
        if retcode != $crate::retcode::Retcode::Okay {
            panic!("{} - SCIP call failed with retcode {:?}", $msg, retcode);
        }
    };
}
