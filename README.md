# russcip

[![tests](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/mmghannam/russcip/actions/workflows/build_and_test.yml)
[![coverage](https://img.shields.io/codecov/c/github/scipopt/russcip)](https://app.codecov.io/gh/scipopt/russcip/)
[![][img_crates]][crates] [![][img_doc]][doc]


[img_crates]: https://img.shields.io/crates/v/russcip.svg

[crates]: https://crates.io/crates/russcip

[img_doc]: https://img.shields.io/badge/rust-documentation-blue.svg

[doc]: https://docs.rs/russcip/

[img_coverage]: https://img.shields.io/codecov/c/github/scipopt/russcip

A safe Rust interface for [SCIP](https://www.scipopt.org/index.php#download). This crate also exposes access to the
SCIP's C-API through the `ffi` module.
The project is currently actively developed, issues/pull-requests are very welcome.

### Installation
The easiest way is to run this in your crate directory
```bash
cargo add russcip --features bundled
```
for other installation methods, please check [INSTALL.md](INSTALL.md).

### Usage

We provide multiple examples listed [here](examples/README.md), and you can also check the [documentation](https://docs.rs/russcip/).

## Accessing unsafe functions

The `ffi` module provides access to the raw C-API of SCIP. This can be used to call functions that are not wrapped in
the safe interface yet.
The `scip_ptr` unsafe function in the `Model` struct, which gives you access to the underlying SCIP raw pointer.
Each other wrapper struct has a similar function named `inner`, e.g. `Variable::inner` or `Constraint::inner` gives you
a `*mut ffi::SCIP_VAR` or `*mut ffi::SCIP_CONS` respectively.

## Implementing Custom Plugins

Some of SCIP's plugins are imported to the rust interface as traits. Currently the implemented plugins are:

| **Name**           | **File**                                                                        | **Docs**                                                                        |
|--------------------|---------------------------------------------------------------------------------|---------------------------------------------------------------------------------|
| Branching rule     | [branchrule.rs](https://github.com/scipopt/russcip/blob/main/src/branchrule.rs) | [docs](https://docs.rs/russcip/latest/russcip/branchrule/trait.BranchRule.html) |
| Variable Pricer    | [pricer.rs](https://github.com/scipopt/russcip/blob/main/src/pricer.rs)         | [docs](https://docs.rs/russcip/latest/russcip/pricer/trait.Pricer.html)         |
| Event Handler      | [eventhdlr.rs](https://github.com/scipopt/russcip/blob/main/src/eventhdlr.rs)   | [docs](https://docs.rs/russcip/latest/russcip/eventhdlr/trait.Eventhdlr.html)   |
| Primal Heuristic   | [heuristic.rs](https://github.com/scipopt/russcip/blob/main/src/heuristic.rs)   | [docs](https://docs.rs/russcip/latest/russcip/heuristic/trait.Heuristic.html)   |
| Separator          | [separator.rs](https://github.com/scipopt/russcip/blob/main/src/separator.rs)   | [docs](https://docs.rs/russcip/latest/russcip/separator/trait.Separator.html)   |
| Constraint Handler | [conshdlr.rs](https://github.com/scipopt/russcip/blob/main/src/conshdlr.rs)     | [docs](https://docs.rs/russcip/latest/russcip/conshdlr/trait.Conshdlr.html)     |

To add a custom plugin to a SCIP `Model` instance, you should implement its trait and call the corresponding
`include_{PLUGIN_NAME}` method. For examples on implementing the specific plugin trait you can check the tests in the
corresponding files.

## Attaching custom data to SCIP instance
This is enabled with the help of the `anymap` crate. You can attach any data to the `Model` instance using the
`set_data` method, and retrieve it using the `get_data` and `get_data_mut` methods.
This is useful for communicating data between plugins, or storing other representations of the
variables/constraints in the model.

## Contributing

Thinking about contributing to _russcip_? First of all thank you! You can check our
issues [page](https://github.com/scipopt/russcip/issues), there's a bunch of [
_good_first_issues_](https://github.com/scipopt/russcip/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22),
or you can check our contribution [guide](CONTRIBUTING.md). If you'd like to contribute and unsure what to do, or
thinking about a big feature and want to discuss if it makes sense and what is the best way to do it? you could open a
new [issue](https://github.com/scipopt/russcip/issues/new/choose)/[discussion](https://github.com/scipopt/russcip/discussions/new/choose)
or send me a quick email [@mmghannam](https://github.com/mmghannam).

## About SCIP

SCIP is currently one of the fastest non-commercial solvers for mixed integer programming (MIP) and mixed integer
nonlinear programming (MINLP). It is also a framework for constraint integer programming and branch-cut-and-price. It
allows for total control of the solution process and the access of detailed information down to the guts of the solver.
