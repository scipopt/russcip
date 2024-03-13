# CHANGELOG

# Unreleased
### Added
### Fixed
### Changed
### Remove

# 0.3.2
### Added
- Support `SCIPfreeTransform` to allow for iterated solving.  

## 0.3.1
### Changed
- Update scip-sys to 0.1.9 to use github hosted binaries of SCIP.

## 0.3.0
### Added
 - Add support for indicator constraints
 - Added method `set_memory_limit` to allow specifying a memory limit.
 - Added `bundled` feature, to download precompiled SCIP binaries. 
### Fixed
 - Fixed Windows MSVC build.

## 0.2.6
### Added
 - Added method `set_obj_integral` to allow specifying that the objective value is always integral.
 - Add support for cardinality constraints.
 - Simplify imports by adding `prelude` module.
### Fixed
### Changed
- Free sol after adding in `ScipPtr::add_sol()`.
- Allow adding a var in `Solving` mode.
- Allow setting parameters from all states.

## 0.2.5
### Added
- Primal heuristic plugin.
- Solving Model state, to represent methods accessible when during solving.
- Moved solution query methods to its own trait.
### Changed
- Moved ScipPtr struct and methods to its own module. 

## 0.2.5
### Added
- Primal heuristic plugin.
- Solving Model state, to represent methods accessible when during solving.
- Moved solution query methods to its own trait.
### Changed
- Moved ScipPtr struct and methods to its own module. 

## 0.2.4
### Added
- Model methods to create child from the current focus node. 
- Node method to get its parent.
- Methods to add set cover and set packing constraints.
- Methods to add sols. 
- Event handler plugin support. 
- Support for quadratic constraints.
### Changed
- Removed the prefix "get_" from all getter methods.
- Move all public methods and types to root of library (to be easily imported).
- Increase coverage.


## 0.2.3
### Added
- Event handler plugin support. 
### Fixed
- Fixed sending plugin data to and from SCIP. 
- Consistent model data across its clones. 


## 0.2.2
### Added
- `ModelRef` to give "unsafe" access to the Model struct to be used in plugins. 
- Add all missing documentation.
- Node wrapper struct. 
- `add_priced_var` method.
- `get_focus_node` method for `ModelRef`. 
### Changed
- Simplified `ModelRef` usage. 


## 0.2.1
### Added
- Solving stats methods for number of nodes, time, and number of lp iterations. 
- Branching rule plugin. 
- Variable pricer plugin. 
- Set partitioning constraints.
### Changed
- Use Variable wrapper in branching candidates. 
