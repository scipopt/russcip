# CHANGELOG

## unreleased
### Added
### Fixed
### Changed
- Free sol after adding in `ScipPtr::add_sol()`.
- Allow adding a var in `Solving` mode.
### Removed

## 0.2.5
### Added
- Primal heuristic plugin.
- Solving Model state, to represent methods accessible when during solving.
- Moved solution query methods to its own trait.
### Fixed
### Changed
- Moved ScipPtr struct and methods to its own module. 
### Removed

## 0.2.4
### Added
- Model methods to create child from the current focus node. 
- Node method to get its parent.
- Methods to add set cover and set packing constraints.
- Methods to add sols. 
- Event handler plugin support. 
- Support for quadratic constraints.
### Fixed
### Changed
- Removed the prefix "get_" from all getter methods.
- Move all public methods and types to root of library (to be easily imported).
- Increase coverage.
### Removed


## 0.2.3
### Added
- Event handler plugin support. 
### Fixed
- Fixed sending plugin data to and from SCIP. 
- Consistent model data across its clones. 
### Changed
### Removed


## 0.2.2
### Added
- `ModelRef` to give "unsafe" access to the Model struct to be used in plugins. 
- Add all missing documentation.
- Node wrapper struct. 
- `add_priced_var` method.
- `get_focus_node` method for `ModelRef`. 
### Fixed
### Changed
- Simplified `ModelRef` usage. 
### Removed


## 0.2.1
### Added
- Solving stats methods for number of nodes, time, and number of lp iterations. 
- Branching rule plugin. 
- Variable pricer plugin. 
- Set partitioning constraints.
### Fixed
### Changed
- Use Variable wrapper in branching candidates. 
### Removed
