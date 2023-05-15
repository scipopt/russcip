//! # russcip
//! Safe Rust interface for SCIP.
//!
//! # Example
//! Model and solve an integer program.
//! ```rust
//! use russcip::model::Model;
//! use russcip::model::ObjSense;
//! use russcip::status::Status;
//! use russcip::variable::VarType;
//! use crate::russcip::model::ModelWithProblem;
//!
//! 
//! // Create model
//! let mut model = Model::new()
//! .hide_output()
//! .include_default_plugins()
//! .create_prob("test")
//! .set_obj_sense(ObjSense::Maximize);
//!
//! // Add variables
//! let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
//! let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);
//!
//! // Add constraints
//! model.add_cons(vec![x1.clone(), x2.clone()], &[2., 1.], -f64::INFINITY, 100., "c1");
//! model.add_cons(vec![x1.clone(), x2.clone()], &[1., 2.], -f64::INFINITY, 80., "c2");
//!
//! let solved_model = model.solve();
//!
//! let status = solved_model.get_status();
//! println!("Solved with status {:?}", status);
//!
//! let obj_val = solved_model.get_obj_val();
//! println!("Objective value: {}", obj_val);
//!
//! let sol = solved_model.get_best_sol().expect("No solution found");
//! let vars = solved_model.get_vars();
//!
//! for var in vars {
//!     println!("{} = {}", &var.get_name(), sol.get_var_val(&var));
//! }

#![deny(missing_docs)]

extern crate doc_comment;
doc_comment::doctest!("../README.md");

/// Re-exports the `scip_sys` crate, which provides low-level bindings to the SCIP library.
pub use scip_sys as ffi;

/// Contains the `BranchRule` trait and several implementations of it.
pub mod branchrule;

/// Contains the `Constraint` struct, which represents a constraint in an optimization problem.
pub mod constraint;

/// The main module, it contains the `Model` struct, which represents an optimization problem.
pub mod model;

/// Contains the `Pricer` trait and several implementations of it.
pub mod pricer;

/// Contains the `Retcode` enum, which represents the return codes of SCIP functions.
pub mod retcode;

/// Contains the `Solution` struct, which represents a solution to an optimization problem.
pub mod solution;

/// Contains the `Status` enum, which represents the status of an optimization problem.
pub mod status;

/// Contains the `Variable` struct, which represents a variable in an optimization problem.
pub mod variable;

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