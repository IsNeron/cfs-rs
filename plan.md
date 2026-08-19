# PLAN.md

# CorrelationFunctions.jl -> Rust migration

## Goal

Turn `cfs-rs` into an idiomatic Rust implementation of
CorrelationFunctions.jl with demonstrated CPU numerical and semantic parity.

The existing Rust repository is a prototype and must be evolved rather than
blindly extended or discarded.

## Definition of CPU parity

CPU parity requires:

* the relevant upstream Julia tests to be represented or otherwise covered;
* deterministic Julia/Rust differential tests;
* all supported dimensions covered;
* all supported directions covered;
* periodic behavior covered;
* non-periodic behavior covered;
* masked behavior covered where supported;
* analytical/invariant tests covered where available;
* no unexplained numerical discrepancies.

API syntax does not need to match Julia.

Scientific behavior does.

---

## M0 — Reference and prototype audit

* [x] Pin the exact CorrelationFunctions.jl upstream commit.
* [x] Add it under `reference/CorrelationFunctions.jl`.
* [x] Record package version and upstream commit.
* [x] Inspect the entire Julia `src/`.
* [x] Inspect the entire Julia test suite.
* [x] Inspect relevant Julia documentation.
* [x] Inspect the complete existing Rust implementation.
* [x] Map Julia public API to implementation files.
* [x] Build a dependency graph of algorithms/primitives.
* [x] Compare existing Rust behavior with Julia behavior.
* [x] Identify semantic bugs in the existing Rust prototype.
* [x] Identify reusable Rust code.
* [x] Identify code that should become a naive/reference implementation.
* [x] Identify missing Julia test coverage or API/documentation inconsistencies.
* [x] Create `docs/reference_audit.md`.
* [x] Update this migration plan if the discovered dependency graph requires it.

No new correlation-function implementation in this milestone.

---

## M1 — Rust project restructuring

**Status: complete.**

Target architecture should approximately separate:

* public library API;
* errors;
* geometry and directions;
* boundary modes;
* slicing;
* normalization;
* transforms;
* morphology;
* directional correlation functions;
* correlation maps.

Convert the repository into a proper library crate.

Move demonstration code to `examples/` if useful.

Preserve useful prototype code.

No new scientific functionality.

---

## M2 — Modes and directions

**Status: complete.**

Implement and validate:

* boundary modes;
* mask representation;
* supported dimensions;
* supported directions;
* direction validation;
* periodic direction restrictions.

Cover every direction defined by the Julia reference.

---

## M3 — Directional slicing

Implement the shared slicing/traversal engine.

Validate independently from correlation functions.

Test:

* 1D;
* 2D;
* 3D;
* axial directions;
* planar diagonals;
* spatial diagonals;
* periodic behavior;
* non-periodic behavior.

Use arrays containing unique coordinate-derived values so traversal order can
be checked exactly.

---

## M4 — Normalization and masks

Reproduce Julia normalization semantics.

Implement and independently validate:

* periodic normalization;
* non-periodic normalization;
* mask-based normalization;
* zero-valid-pair behavior;
* lag conventions.

---

## M5 — FFT correlation primitives

Implement internal:

* autocorrelation;
* cross-correlation.

First compare FFT implementations against naive O(N^2) implementations.

Explicitly handle inverse FFT normalization.

No public S2 yet.

---

## M6 — S2 and cross-correlation

Implement directional:

* S2;
* cross-correlation.

Validate using:

* Julia golden fixtures;
* exact small cases;
* seeded random inputs;
* all dimensions;
* all directions;
* all boundary modes supported by Julia.

---

## M7 — L2

Implement lineal-path correlation.

Validate:

* periodic run merging;
* non-periodic runs;
* analytical/simple cases;
* Julia fixtures;
* scientific invariants.

---

## M8 — Connected components and C2

Extract connected-component labeling into shared morphology infrastructure.

Match Julia connectivity exactly.

Validate connected components independently.

Then implement production C2.

Retain the old Rust C2 implementation as a slow/reference implementation if
useful.

Compare:

* Julia C2;
* naive Rust C2;
* production Rust C2.

---

## M9 — Euclidean distance transform

Port Julia EDT semantics.

Validate against:

* Julia output;
* brute-force distance calculations;
* 1D;
* 2D;
* 3D;
* periodic;
* non-periodic.

---

## M10 — Pore size and chord length

Implement:

* pore-size statistics;
* chord-length statistics.

Reuse EDT and slicing primitives.

Validate against upstream analytical and geometry-based tests.

---

## M11 — Surface extraction

Port the Julia surface/edge extraction algorithms exactly.

Support the reference kernels.

Validate surface fields independently before implementing surface correlation
functions.

---

## M12 — Surface correlation functions

Implement:

* Fss;
* Fsv;
* related two-point surface functions exposed by the reference.

Validate against analytical cases and Julia fixtures.

---

## M13 — Three-point functions

Implement shared shift/pattern infrastructure.

Then implement:

* S3;
* C3;
* three-point surface functions.

Validate invariants such as reduction to two-point functions when one lag is
zero where applicable.

---

## M14 — Correlation maps

Implement N-dimensional map variants.

Validate:

* Julia map fixtures;
* directional values extracted from maps against independently computed
  directional functions.

---

## M15 — Remaining utility API

Port remaining scientifically relevant utility functions from Julia.

Do not port utilities merely because they exist if they are obsolete or
internal and unnecessary; document such decisions.

---

## M16 — Full CPU parity audit

Perform a complete comparison against the pinned Julia reference.

Produce a feature-parity matrix.

No unexplained differences may remain.

---

## M17 — Benchmarks and optimization

Only after CPU parity:

* benchmark Julia versus Rust;
* benchmark naive versus optimized Rust;
* profile;
* introduce Rayon where useful;
* reduce allocations;
* evaluate FFT planning/caching;
* consider SIMD where justified.

Behavior must remain covered by parity tests.

---

## M18 — Python bindings

Add a separate Python-facing layer using an appropriate Rust/Python binding
stack.

Do not compromise the core Rust API to mimic Python.

---

## M19 — CUDA backend

Design and implement GPU acceleration only after the CPU implementation is
stable.

GPU results must be compared against CPU Rust and Julia reference results.

---

## Current milestone

M2 — Modes and directions (complete). M3 has not started.
