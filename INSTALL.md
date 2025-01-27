## Installation

By running

```bash
cargo add russcip --features bundled
```

The `bundled` feature will download a precompiled SCIP as part of the build process.
This is the easiest to get started with russcip.

### Custom SCIP installation

If no feature is not enabled, `russcip` will look for a scip installation in the current conda environment,
to install SCIP using conda run the following command

```bash
conda install --channel conda-forge scip
```

Alternatively, you can specify the installation directory through the `SCIPOPTDIR` environment variable.

*russcip* is tested against SCIP 9.0.0 but it might work for other versions depending on which functionality you use.


### `from-source` feature
To build SCIP from source, you can enable the `from-source` feature. This will download the SCIP source code and build
it as part of the build process.

```bash
cargo add russcip --features from-source
```
This is currently the most reliable way to get a statically-linked build of SCIP. However, it only includes SCIP with
SoPlex so it can only handle linear constraints.
