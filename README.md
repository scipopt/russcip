# russcip
[![tests](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml)
[![][img_crates]][crates] [![][img_doc]][doc]

[img_crates]: https://img.shields.io/crates/v/russcip.svg
[crates]: https://crates.io/crates/russcip
[img_doc]: https://img.shields.io/badge/rust-documentation-blue.svg
[doc]: https://docs.rs/russcip/

A safe Rust interface for [SCIP](https://www.scipopt.org/index.php#download). This crate also exposes access to the SCIP's C-API through the `ffi` module. 
The project is currently an early-stage work in progress, issues/pull-requests are very welcome. 
## Dependencies 
Make sure SCIP is installed, the easiest way to install it is to install a precompiled package from [here](https://scipopt.org/index.php#download) or through conda by running
```bash
conda install --channel conda-forge scip
```
After which `russcip` would be able to find the installation in the current Conda environment. Alternatively, you can specify the installation directory through the `SCIPOPTDIR` environment variable. 

## Install
By running
```bash
cargo add russcip
```
or to get the most recent version, add the following to your `Cargo.toml`
```toml
[dependencies]
russcip = { git = "https://github.com/mmghannam/russcip" }
```

## Example
Model and solve an integer program.
```rust
use russcip::model::Model;
use russcip::model::ObjSense;
use russcip::status::Status;
use russcip::variable::VarType;
use russcip::retcode::Retcode;

fn main() -> Result<(), Retcode> {
    // Create model
    let mut model = Model::new()?;
    model.include_default_plugins()?;
    model.create_prob("test")?;
    model.set_obj_sense(ObjSense::Maximize)?;
    model.hide_output()?;

    // Add variables
    let x1_id = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer)?;
    let x2_id = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer)?;

    // Add constraints
    model.add_cons(&[x1_id, x2_id], &[2., 1.], -f64::INFINITY, 100., "c1")?;
    model.add_cons(&[x1_id, x2_id], &[1., 2.], -f64::INFINITY, 80., "c2")?;

    model.solve()?;

    let status = model.get_status();
    println!("Solved with status {:?}", status);

    let obj_val = model.get_obj_val();
    println!("Objective value: {}", obj_val);

    let sol = model.get_best_sol().unwrap();
    let vars = model.get_vars();

    for var in vars {
        println!("{} = {}", &var.get_name(), sol.get_var_val(&var));
    }

    Ok(())
}

```

## About SCIP

SCIP is currently one of the fastest non-commercial solvers for mixed integer programming (MIP) and mixed integer nonlinear programming (MINLP). It is also a framework for constraint integer programming and branch-cut-and-price. It allows for total control of the solution process and the access of detailed information down to the guts of the solver.
