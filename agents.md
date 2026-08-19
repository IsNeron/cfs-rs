# AGENTS.md

## Project goal

This repository is a Rust reimplementation of:

https://github.com/fatimp/CorrelationFunctions.jl

The primary goal is numerical and semantic compatibility with the pinned
Julia reference implementation.

Correctness, reproducibility, and scientific equivalence are more important
than performance during the migration.

This repository already contains an earlier Rust prototype. Treat that code
as potentially useful, but do not assume that its behavior matches Julia.

## Reference implementation

The Julia reference implementation must be kept under:

```
reference/CorrelationFunctions.jl/
```

The exact upstream commit must be pinned and documented.

Treat the reference implementation as read-only.

Do not modify Julia source code or Julia tests to make the Rust implementation
appear compatible.

When behavior is unclear:

1. inspect the Julia implementation;
2. inspect Julia tests;
3. inspect Julia documentation;
4. determine actual behavior;
5. reproduce that behavior in Rust.

## Existing Rust prototype

The repository contains an existing partial Rust implementation.

Do not delete working code merely because the architecture is being changed.

Existing code may be:

* retained;
* moved;
* refactored;
* reused as a naive/reference implementation;
* replaced only after equivalent behavior is covered by tests.

In particular, existing C2 code should be treated as a useful prototype and
potential slow reference implementation until semantic parity is established.

## Migration strategy

Work only on the currently requested milestone.

Do not implement later milestones preemptively.

The migration order is:

1. audit reference and existing Rust code;
2. establish project architecture;
3. migrate shared primitives;
4. establish differential tests;
5. migrate scientific functions;
6. establish full CPU parity;
7. benchmark and optimize;
8. add bindings and GPU support.

Do not translate Julia syntax mechanically.

Design idiomatic Rust APIs while preserving scientific semantics.

## Scientific compatibility

Do not silently change:

* definitions of correlation functions;
* normalization;
* boundary conditions;
* connectivity;
* masks;
* dimensional conventions;
* directional conventions;
* surface definitions;
* FFT normalization;
* distance definitions;
* indexing semantics.

Any intentional difference from Julia must be explicitly documented.

## Numerical parity

For every migrated algorithm:

1. add deterministic unit tests;
2. add Julia-derived reference/golden tests where appropriate;
3. test analytical invariants when available;
4. compare floating-point values using justified tolerances;
5. test periodic and non-periodic modes separately;
6. test masked behavior when supported;
7. test every supported dimensionality;
8. test every supported direction.

Never loosen a tolerance merely to make a test pass.

Investigate the discrepancy first.

## Differential testing

The preferred validation model is:

```
same input
   |
   +--> Julia reference --> expected output
   |
   +--> Rust implementation --> actual output
```

Julia-generated fixtures should be deterministic.

Random fixtures must use fixed seeds.

Where practical, also maintain simple naive Rust implementations as an
independent oracle for optimized implementations.

## Array conventions

Be explicit about:

* dimension order;
* axis order;
* memory order;
* Julia 1-based versus Rust 0-based indexing;
* lag/distance indexing;
* padding;
* periodic wrapping;
* masks.

Do not depend implicitly on flattened memory order when logical
multidimensional indexing can be used.

## FFT

FFT implementations must explicitly account for forward/inverse
normalization differences between Julia FFT libraries and Rust FFT libraries.

Every FFT correlation primitive must first be tested against a naive
O(N^2) implementation on small arrays.

## Connected components

Connectivity must match the Julia implementation exactly.

Do not substitute a third-party connected-component implementation until
reference parity has been demonstrated.

Periodic connected components must be tested explicitly.

Masked voxels must not participate in components unless that matches the
Julia reference behavior.

## EDT and surface extraction

Do not replace the Julia Euclidean distance transform or surface extraction
algorithms with approximate third-party implementations during the parity
phase.

First reproduce and test the reference behavior.

Alternative implementations may be considered only after parity exists.

## Performance

Do not optimize prematurely.

During the parity phase:

* prefer clear sequential implementations;
* do not introduce unsafe Rust;
* do not introduce SIMD-specific code;
* do not introduce Rayon unless requested by the milestone;
* do not add CUDA.

Performance work begins only after the relevant behavior is covered by tests.

## GPU and bindings

Do not add:

* CUDA;
* Python bindings;
* R bindings;
* Julia bindings;
* CLI infrastructure beyond what is required for testing;

unless the active milestone explicitly requests them.

## Validation

Before completing any implementation milestone run:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

If reference Julia tests are part of the milestone, run them as well.

Do not proceed to another milestone while required validation is failing.

## Changes

Prefer small, reviewable changes.

Do not combine:

* architectural restructuring;
* new scientific algorithms;
* performance optimization;

in the same milestone unless explicitly requested.

## Reporting

At the end of every task report:

* files created;
* files modified;
* files moved;
* Julia reference files inspected;
* behavior implemented or audited;
* tests added;
* validation commands run;
* exact test results;
* known differences from Julia;
* unresolved questions;
* recommended next milestone.

Never claim feature parity unless tests demonstrate it.
