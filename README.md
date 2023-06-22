# russcip
[![tests](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml)
[![coverage](https://img.shields.io/codecov/c/github/scipopt/russcip)](https://app.codecov.io/gh/scipopt/russcip/)
[![][img_crates]][crates] [![][img_doc]][doc] 





[img_crates]: https://img.shields.io/crates/v/russcip.svg
[crates]: https://crates.io/crates/russcip
[img_doc]: https://img.shields.io/badge/rust-documentation-blue.svg
[doc]: https://docs.rs/russcip/
[img_coverage]: https://img.shields.io/codecov/c/github/scipopt/russcip

A safe Rust interface for [SCIP](https://www.scipopt.org/index.php#download). This crate also exposes access to the SCIP's C-API through the `ffi` module. 
The project is currently an early-stage work in progress, issues/pull-requests are very welcome. 
## Dependencies 
Make sure SCIP is installed, the easiest way to install it is to install a precompiled package from [here](https://scipopt.org/index.php#download) or through conda by running
```bash
conda install --channel conda-forge scip
```
After which `russcip` would be able to find the installation in the current Conda environment. Alternatively, you can specify the installation directory through the `SCIPOPTDIR` environment variable. 

*russcip* is tested against SCIP 8.0.3 but it might work for other versions depending on which functionality you use. 

## Installation
By running
```bash
cargo add russcip
```
or to get the most recent version, add the following to your `Cargo.toml`
```toml
[dependencies]
russcip = { git = "https://github.com/scipopt/russcip" }
```

## Example
Model and solve an integer program.
```rust
use russcip::model::Model;
use russcip::model::ObjSense;
use russcip::status::Status;
use russcip::variable::VarType;
use russcip::retcode::Retcode;
use crate::russcip::model::ModelWithProblem;

fn main() {
    // Create model
    let mut model = Model::new()
    .hide_output()
    .include_default_plugins()
    .create_prob("test")
    .set_obj_sense(ObjSense::Maximize);

    // Add variables
    let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
    let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);

    // Add constraints
    model.add_cons(vec![x1.clone(), x2.clone()], &[2., 1.], -f64::INFINITY, 100., "c1");
    model.add_cons(vec![x1.clone(), x2.clone()], &[1., 2.], -f64::INFINITY, 80., "c2");

    let solved_model = model.solve();

    let status = solved_model.status();
    println!("Solved with status {:?}", status);

    let obj_val = solved_model.obj_val();
    println!("Objective value: {}", obj_val);

    let sol = solved_model.best_sol().unwrap();
    let vars = solved_model.vars();

    for var in vars {
        println!("{} = {}", &var.name(), sol.val(var));
    }
}

```

## The `raw` feature
You can enable this feature by specifying the feature in your `Cargo.toml`
```toml
[dependencies]
russcip = { features = ["raw"] }
```
This enables access to the `scip_ptr` unsafe function in the `Model` struct, which gives you access to the underlying SCIP raw pointer. This is can be used in combination with the `ffi` module to call SCIP functions that are not wrapped yet in the safe interface. 

## Implementing Custom Plugins
Some of SCIP's plugins are imported to the rust interface as traits. Currently the implemented plugins are: 

|   **Name**    |                          **File**                          |                                                                                              **Docs**                                                                                              |
|---------------|------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Branching rule| [branchrule.rs](https://github.com/scipopt/russcip/blob/main/src/branchrule.rs) | [docs](https://docs.rs/russcip/latest/russcip/branchrule/trait.BranchRule.html) |
| Variable Pricer| [pricer.rs](https://github.com/scipopt/russcip/blob/main/src/pricer.rs) | [docs](https://docs.rs/russcip/latest/russcip/pricer/trait.Pricer.html) |
| Event Handler | [eventhdlr.rs](https://github.com/scipopt/russcip/blob/main/src/eventhdlr.rs) | [docs](https://docs.rs/russcip/latest/russcip/eventhdlr/trait.Eventhdlr.html) |

To add a custom plugin to a SCIP `Model` instance, you should implement its trait and call the corresponding `include_{PLUGIN_NAME}` method. For examples on implementing the specific plugin trait you can check the tests in the corresponding files. 

## About SCIP

SCIP is currently one of the fastest non-commercial solvers for mixed integer programming (MIP) and mixed integer nonlinear programming (MINLP). It is also a framework for constraint integer programming and branch-cut-and-price. It allows for total control of the solution process and the access of detailed information down to the guts of the solver.
