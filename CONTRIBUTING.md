# Contributing to _russcip_

First off, thank you for considering contributing to russcip.

## Where do I go from here?

If you've noticed a bug or have a feature request, make sure to open a GitHub issue if one does not already exist. 

## Porting part of the SCIP C-API to safe Rust

Steps: 
1. Add a method to the `ScipPtr` struct, this should in most cases return a `Result<YOU_RETURN_TYPE, Retcode>` from calling the `scip_call!` macro. Use the unsafe bindings from the `ffi` module to call SCIP methods.
2. The Model<T> struct has many [states](https://docs.rs/russcip/latest/russcip/model/index.html), choose the state that is safe to call your `ScipPtr` method in.


## Wrapping a SCIP plugin

1. SCIP plugins are essentially interfaces. So first step would be to check the plugin's (fundamental) callbacks from SCIP's documentation and write an interface that represents it. Here's the [one](https://www.scipopt.org/doc/html/CONS.php) for the constraint handler plugin for example. 
2. SCIP saves PLUGIN_DATA for each plugin type. This is what _russcip_ uses to wrap the struct that implements the interface. You'd need to add an `include_{PLUGIN_NAME}` method on `Model<ProblemCreated>` that takes a box of this interface and defines the C callbacks. As a reference you can take a look at the pricer plugin [implementation](https://github.com/scipopt/russcip/blob/main/src/pricer.rs) and it's include [method](https://github.com/scipopt/russcip/blob/e13d20d3e8594d3262f312304f4f2400de48da39/src/scip.rs#L703). 


## Code Quality 

- Add documentation for any new methods/struct.
- Cover any new functionality or test any issue fix . 
- Update the [change log](https://github.com/scipopt/russcip/blob/main/CHANGELOG.md).
