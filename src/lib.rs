//! # russcip
//! Safe Rust interface for [SCIP](https://scipopt.org/) optimization suite.
//!
//! For examples and usage, please refer to the [repository](https://github.com/scipopt/russcip).

#![deny(missing_docs)]
#![allow(clippy::macro_metavars_in_unsafe)]

/// Re-exports the `scip_sys` crate, which provides low-level bindings to the SCIP library.
pub use scip_sys as ffi;

/// Contains the `BranchRule` trait used to define custom branching rules.
pub mod branchrule;
pub use branchrule::*;

/// Contains the `Constraint` struct, which represents a constraint in an optimization problem.
pub mod constraint;
pub use constraint::*;

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
