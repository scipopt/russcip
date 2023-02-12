# russcip
[![tests](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml)

A safe Rust interface for [SCIP](https://www.scipopt.org/index.php#download). The project is currently an early-stage work in progress, issues/pull-requests are very welcome. 

This crate also exposes access to the SCIP's C-API through the `ffi` module. 

## Dependencies 
make sure [SCIP](https://www.scipopt.org/index.php#download) 8.0.3 is installed and included in the library path, or define an environment variable `SCIPOPTDIR` with the install directory. 

## Install
By running
```bash
cargo add russcip
```
or to get the most recent version, add the following to your `Cargo.toml`
```
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

fn main() {
    // Create model
    let mut model = Model::new();
    model.include_default_plugins();
    model.create_prob("test");
    model.set_obj_sense(ObjSense::Maximize);
    model.hide_output();

    // Add variables
    let x1 = model.add_var(0., f64::INFINITY, 3., "x1", VarType::Integer);
    let x2 = model.add_var(0., f64::INFINITY, 4., "x2", VarType::Integer);

    // Add constraints
    model.add_cons(&[&x1, &x2], &[2., 1.], -f64::INFINITY, 100., "c1");
    model.add_cons(&[&x1, &x2], &[1., 2.], -f64::INFINITY, 80., "c2");

    model.solve();

    let status = model.get_status();
    println!("Solved with status {:?}", status);

    let obj_val = model.get_obj_val();
    println!("Objective value: {}", obj_val);

    let sol = model.get_best_sol();
    let vars = model.get_vars();

    for var in vars {
        println!("{} = {}", &var.get_name(), sol.get_var_val(&var));
    }
}

```


