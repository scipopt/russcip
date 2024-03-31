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
The project is currently actively developed, issues/pull-requests are very welcome.

## Installation

By running
```bash
cargo add russcip --features bundled
```

The `bundled` feature will download a precompiled SCIP as part of the build process.
This is the easiest to get started with russcip, instead you could try the [from-source](#from-source-feature) 
or if you want to link against your custom SCIP installation check the [section](#custom-scip-installation) below.

### `from-source` feature
To build SCIP from source, you can enable the `from-source` feature. This will download the SCIP source code and build it as part of the build process. 
```bash
cargo add russcip --features from-source
```

### Custom SCIP installation
If no feature is not enabled, `russcip` will look for a scip installation in the current conda environment,
to install SCIP using conda run the following command
```bash
conda install --channel conda-forge scip
```
Alternatively, you can specify the installation directory through the `SCIPOPTDIR` environment variable.

*russcip* is tested against SCIP 9.0.0 but it might work for other versions depending on which functionality you use. 


### Examples
An [example](examples/create_and_solve.rs) on how to model and solve an integer program can be found in the [examples](examples) directory.
To run the example, you can use the following command
```bash
cargo run --example create_and_solve
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
| Primal Heuristic | [heuristic.rs](https://github.com/scipopt/russcip/blob/main/src/heuristic.rs) | [docs](https://docs.rs/russcip/latest/russcip/eventhdlr/trait.Heuristic.html) |

To add a custom plugin to a SCIP `Model` instance, you should implement its trait and call the corresponding `include_{PLUGIN_NAME}` method. For examples on implementing the specific plugin trait you can check the tests in the corresponding files. 

## Contributing
Thinking about contributing to _russcip_? First of all thank you! You can check our issues [page](https://github.com/scipopt/russcip/issues), there's a bunch of [_good_first_issues_](https://github.com/scipopt/russcip/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22), or you can check our contribution [guide](CONTRIBUTING.md). If you'd like to contribute and unsure what to do, or thinking about a big feature and want to discuss if it makes sense and what is the best way to do it? you could open a new [issue](https://github.com/scipopt/russcip/issues/new/choose)/[discussion](https://github.com/scipopt/russcip/discussions/new/choose) or send me a quick email [@mmghannam](https://github.com/mmghannam).

## About SCIP

SCIP is currently one of the fastest non-commercial solvers for mixed integer programming (MIP) and mixed integer nonlinear programming (MINLP). It is also a framework for constraint integer programming and branch-cut-and-price. It allows for total control of the solution process and the access of detailed information down to the guts of the solver.
