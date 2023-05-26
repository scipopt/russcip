# CHANGELOG

## Unreleased
### Added
- Event handler plugin support. 
### Fixed
- Fixed sending plugin data to and from SCIP. 
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
