# M0 reference and prototype audit

## Scope and status

This document audits the Julia reference and the pre-existing Rust prototype. It does not implement or repair any scientific function. The audit covers every file under Julia `src/`, every Julia test file, the package and test projects, the shipped Markdown documentation, and every Rust source and test in this repository.

The Julia source is included as a read-only git submodule at `reference/CorrelationFunctions.jl`. A submodule was chosen instead of copying source because the parent repository records an exact gitlink while preserving upstream history and making accidental edits visible through `git submodule status` and `git -C reference/CorrelationFunctions.jl status`.

## 1. Pinned Julia reference

| Item | Audited value |
| --- | --- |
| Upstream | `https://github.com/fatimp/CorrelationFunctions.jl.git` |
| Commit | `c76f1625368ae4a1c6326b9055e51ddcc9bcf19e` |
| Commit date | 2026-04-27 |
| `Project.toml` version | `0.14.0` |
| Nearest tag description | `v0.14.0-5-gc76f162` |
| Julia compatibility | `^1.6` |

The upstream tree has no `Manifest.toml`, so exact transitive dependency versions are not pinned by upstream. The declared direct compatibility ranges are CUDA `3.2, 4, 5`, CircularArrays `1.2`, FFTW `1.4`, ImageFiltering `0.7.1`, ImageMorphology `0.4.5`, OffsetArrays `1`, Statistics `1`, and LinearAlgebra `1.10.0`. The test project additionally declares CUDA, Distributions, ImageFiltering, LinearAlgebra, StatsBase, and XUnit without compat bounds. Reproducing a resolved environment therefore also requires preserving a Julia registry snapshot or generating a reviewed manifest in a later milestone. No manifest was generated in M0 because Julia is unavailable in this environment and the reference tree must remain untouched.

## 2. Julia public API inventory

The root module exports `Directional`, `Map`, `Utilities`, and re-exports `pore_size` for historical reasons. Export declarations are in `src/CorrelationFunctions.jl`.

### Directional scientific API

Directions below mean the direction types valid for the stated rank: 1D `DirX`; 2D `DirX`, `DirY`, `DirXY`, `DirYX`; 3D all 13 exported directions. The 3D set is axial `DirX/Y/Z`, planar `DirXY/YX/XZ/ZX/YZ/ZY`, and spatial `DirXYZ/XZY/YXZ/ZYX`.

| Public function | Source | Dimensions and directions | Modes | Principal dependencies |
| --- | --- | --- | --- | --- |
| `s2` | `src/directional/s2.jl` | 1D-3D, rank-valid directions | periodic, non-periodic, mask | phase indicator, mask, slicer, FFT plan/autocorrelation, normalization |
| `cross_correlation` | `src/directional/cc.jl` | 1D-3D, rank-valid directions | periodic, non-periodic, mask | two indicators, mask, slicer, FFT cross-correlation, normalization |
| `l2` | `src/directional/l2.jl` | 1D-3D, rank-valid directions | periodic, non-periodic, mask | mask, slicer, run counting/periodic run merge, normalization |
| `c2` | `src/directional/c2.jl` | intended 1D-3D and rank-valid directions; validation is incomplete | periodic, non-periodic, mask | indicator, mask, connected components, slicer, normalization; no FFT |
| `pore_size` | `src/directional/pore-size.jl` | 1D-3D, not directional | periodic, non-periodic; `Mask` is accepted by type but behaves as non-periodic EDT | indicator complement, EDT |
| `chord_length` | `src/directional/chord-length.jl` | 1D-3D, rank-valid directions | fixed non-periodic | indicator, EDT surface (`distance == 1`), slicer, edge-to-edge scan |
| `surf2` | `src/directional/surface.jl` | practically 2D-3D, rank-valid directions | periodic, non-periodic, mask | mask, surface extraction, slicer, FFT autocorrelation, normalization |
| `surfvoid` | `src/directional/surface.jl` | declared rank >=1; default convolution filter is practically 2D-3D | periodic, non-periodic, mask | void indicator, mask, surface extraction, FFT cross-correlation, normalization |
| `s3` | `src/directional/s3.jl` | generic N-D arrays with N-tuple shift patterns; tests use 3D | periodic, non-periodic | zero-padded/circular shifts, ternary product, direct normalization; no FFT |
| `c3` | `src/directional/c3.jl` | as `s3`; connected-component helper itself is supported only for 1D-3D in periodic mode | periodic, non-periodic | indicator, connected components, `autocorr3_plane` |
| `surf3` | `src/directional/surface3.jl` | requires rank >=3; surface filter is implemented for 2D/3D | periodic, non-periodic | surface extraction, `autocorr3_plane` |
| `surf2void` | `src/directional/surface3.jl` | requires rank >=2; practically 2D/3D surface extraction | periodic, non-periodic | surface extraction, void indicator, `crosscorr3_plane` |
| `surfvoid2` | `src/directional/surface3.jl` | declared rank >=1; practically 2D/3D surface extraction | periodic, non-periodic | surface extraction, void indicator, `crosscorr3_plane` |

Three-point functions do not use `AbstractDirection`; callers supply two broadcast-compatible arrays of N-dimensional integer shift tuples, commonly created with `right_triangles`.

### Correlation-map API

| Public function | Source | Dimensions/directions | Modes | Principal dependencies |
| --- | --- | --- | --- | --- |
| `Map.s2` | `src/map/s2_map.jl` | advertised N-D map | periodic, non-periodic, mask | mask, N-D padding, FFT autocorrelation, map normalization |
| `Map.cross_correlation` | `src/map/cc_map.jl` | advertised N-D map | periodic, non-periodic, mask | mask, padding, N-D FFT cross-correlation, map normalization |
| `Map.c2` | `src/map/c2_map.jl` | 1D-3D for periodic labeling; broader non-periodic behavior depends on ImageMorphology | mandatory `mode` keyword despite documented default | mask, connected components, per-component FFT autocorrelation, map normalization |
| `Map.surf2` | `src/map/surface.jl` | practically 2D-3D | periodic, non-periodic, mask | mask, surface extraction, N-D FFT autocorrelation, map normalization |
| `Map.surfvoid` | `src/map/surface.jl` | practically 2D-3D | periodic, non-periodic, mask | mask, surface extraction, N-D FFT cross-correlation, map normalization |
| `dir_from_map` | `src/map/misc.jl` | methods only for X, Y, Z, XY, YX; implementations hard-code 3-coordinate indices | periodic/non-periodic length rule | direct map indexing |
| `average_directions` | `src/map/misc.jl` | generic N-D map | not mode-aware | rounded Euclidean radial bins |

### Public utilities and designators

`Utilities` exports:

- data/numerical utilities: `read_cuboid` (`utility/rawreader.jl`), `lowfreq_energy_ratio` (`utility/lowfreq_energy_ratio.jl`), `edt`, `label_components`, and `extract_edges` (`utility/images.jl`);
- surface kernels: `AbstractKernel`, `ConvKernel`, `ErosionKernel` (`utility/images.jl`);
- boundary types and helpers: `AbstractMode`, `Periodic`, `NonPeriodic`, `Mask`, `maybe_add_padding`, `maybe_apply_mask` (`utility/modes.jl`);
- direction types and validation: `AbstractDirection`, all 13 `Dir*` types, `check_direction` (`utility/directions.jl`);
- three-point patterns: `RightTrianglePattern`, `AbstractPlane`, `PlaneXY/XZ/YZ`, `right_triangles` (`utility/pattern.jl`);
- integration helpers: `check_rank` and `maybe_upload_to_gpu` (`utility/misc.jl`).

`edt` uses an exact separable squared-distance lower-envelope transform. Periodic 1D passes triple-concatenated data through the non-periodic transform; 2D and 3D apply the 1D transform successively along axes. `label_components` uses face/Manhattan connectivity: custom traversal in 1D and for all periodic arrays, and ImageMorphology for non-periodic 2D/3D. `extract_edges` uses either normalized convolution with reflected/circular padding or spherical-kernel erosion.

## 3. Actual implementation dependency graph

The graph below is derived from includes and calls in `src/`, not from `PLAN.md`.

```text
AbstractMode + Mask ----> maybe_apply_mask / maybe_add_padding
          |                         |
          |                         +--> mask-pair normalization
          +--> periodic/non-periodic slicer behavior

AbstractDirection + rank/shape ----> check_direction
          |
          +--> slices/diagonals ----> normalization
                         |           |       |
                         |           |       +--> L2
                         |           +--> FFT autocorr/crosscorr --> S2 / cross-correlation
                         +--> C2 pair traversal
                         +--> chord scan

phase indicator --> maybe_apply_mask --> label_components --> C2
                                                |
                                                +--> C3
                                                +--> Map.C2 (per-label FFTs)

phase complement --> exact EDT --> pore_size
phase indicator  --> exact EDT --> distance==1 surface --> chord_length

phase indicator --> extract_edges --> FFT correlation --> directional/map surf2/surfvoid
                                      |
                                      +--> direct 3-point shifts --> surf3/surf2void/surfvoid2

shift patterns --> direct pad/circular shifts --> S3
                                      |
label_components ---------------------+--> C3

N-D padding + FFT + map normalization --> Map.S2 / Map.cross-correlation
label_components + per-label FFT ------> Map.C2
extract_edges + N-D FFT ---------------> surface maps
map output ----------------------------> dir_from_map / average_directions
```

Specific ordering facts:

1. Modes, masks, direction validation, and directional slicing are shared foundations.
2. Directional normalization depends on slicing; masked normalization additionally depends on FFT autocorrelation of the mask.
3. FFT planning depends on direction validation, slicing, and padding. S2 and directional cross-correlation depend on both FFT correlation and normalization.
4. L2 is FFT-free but depends on slicing, modes, run counting, periodic end-run merging, and normalization.
5. Connected components precede C2, C3, and the C2 map. C2 itself is direct pair counting over labeled slices, not FFT-based; the C2 map is FFT-based per component.
6. EDT precedes both pore size and chord length. Chord length also depends on slicing; its "surface" is `edt(indicator) == 1`, not `extract_edges`.
7. Surface extraction precedes all two- and three-point surface functions.
8. Three-point S3 is a direct shift engine. C3 adds connected components; three-point surface functions add edge extraction.
9. Correlation maps use a separate N-D FFT and normalization path, then optionally feed `dir_from_map` or `average_directions`.

This ordering is compatible with the broad `PLAN.md` sequence. In particular, EDT is already before chord length, connected components are introduced with C2 before C3, and surface extraction is before surface functions. No milestone reorder is justified, so `PLAN.md` is unchanged. M1 should nevertheless preserve the two distinct correlation engines: directional sliced transforms and N-D map transforms.

## 4. Julia test-suite inventory

| File | Nature of coverage |
| --- | --- |
| `test/short.jl` | exact 1D S2/L2 values, periodic/non-periodic cases, and all-one invariants for S2/L2/C2 |
| `test/random.jl` | probabilistic and invariant tests for S2/L2/C2/surface functions, cross-correlation identity, S3-to-S2 and C3-to-C2 reductions, negative three-point shifts |
| `test/checkboard.jl` | exact periodic checkerboard S2/L2/C2 behavior along axial directions |
| `test/value-noise.jl` | reflection invariance and mask-vs-cropped-array comparisons for directional/map S2, C2, cross-correlation, and surface functions |
| `test/disks.jl` | 2D analytical/geometric tests for L2, S2, surface functions, pore-size distribution, chord lengths |
| `test/balls.jl` | 3D analytical/geometric tests plus three-point surface reductions |
| `test/square.jl`, `test/cube.jl` | surface-filter rotation behavior and an Fsss exact geometry case |
| `test/supplementary.jl` | brute-force 2D/3D EDT checks for both boundary modes, low-frequency energy, map radial averaging |
| `test/maps.jl` | map-vs-directional comparisons for S2/C2 and surface functions by rank/mode |
| `test/gpu.jl` | opt-in CPU/GPU comparisons, currently stale |
| `test/utility.jl` | generators, analytical helpers, direction lists, numerical helpers |

Coverage types include exact values (`short`, `checkboard`), analytical comparisons (`disks`, `balls`, `square`, `cube`), invariants/properties (`random`, `value-noise`, `supplementary`), random tests, map-vs-directional differential tests, and opt-in GPU tests. Random fixtures are generally not seeded; several generators draw new random seeds internally, so failures are not fully reproducible.

Direction coverage has a concrete gap: `known_directions` contains 12 of the 13 public 3D directions and omits `DirXZY`. Axial directions receive much stronger analytical and mask coverage than diagonals. `test/maps.jl` checks all four 2D directions but only X/Y/Z in 3D. Mask tests use `DirX` only. There is no focused slicing test that verifies coordinate order for every direction.

Mode coverage is broad for S2/L2/C2 and maps, but three-point functions cover only periodic and non-periodic modes; `Mask` is not supported by their typed methods. EDT is tested in 2D/3D for periodic and non-periodic modes, not 1D. Periodic connected components are only indirectly exercised through C2.

Test and documentation issues:

- `test/gpu.jl` uses the removed `periodic` keyword even though version 0.13 replaced it with `mode`; the GPU suite is opt-in and therefore normally skipped.
- `examples/notebook.ipynb` also uses the removed `periodic` keyword throughout; `examples/3points.ipynb` uses the current `mode` API.
- `Map.c2` documentation promises `mode = NonPeriodic()`, but its method signature requires the keyword.
- `docs/src/index.md` says correlation functions support 1D-3D, while the README says 2D/3D and surface convolution is only defined meaningfully for 2D/3D.
- Chord-length documentation/results discuss histogram bins even though version 0.11 changed the API to return raw lengths; the tests build histograms externally except for stale prose saying `nbins` was passed to `chord_length`.
- Directional surface docs say a function-valued `phase` is applied, but implementations use `array .== phase`; no predicate dispatch exists.
- `dir_from_map` is documented generically, but supports only five directions and constructs 3-coordinate `CartesianIndex` values. This is inconsistent with the 1D/2D calls in `test/maps.jl` and needs confirmation in a runnable Julia environment.
- `known_directions` omits `DirXZY`, leaving one declared direction untested.
- The source comment in `edge_filter` computes `sqradius` without using it in the convolution method; this is harmless but stale.

The attempted upstream CPU command was:

```powershell
cd reference/CorrelationFunctions.jl
julia --project=. -e "using Pkg; Pkg.test()"
```

It did not start: PowerShell reported `CommandNotFoundException` because `julia` is not installed or on `PATH`. Therefore M0 records source-level test coverage but cannot claim that the pinned upstream suite passes.

## 5. Existing Rust code inventory

The crate is currently a binary-only prototype named `crf_rs`, using ndarray, rustfft, and thiserror. `src/main.rs` is a demonstration driver; modules are private below `src/core`. All seven Rust tests are inline in `src/core/directional/mod.rs`.

| Rust code | Classification | Audit finding |
| --- | --- | --- |
| `Direction` and `direction_step` in `src/core/directions.rs` | reusable after refactoring | All 13 Julia vectors are represented correctly. `Diagonal` is an extra ambiguous alias. Validation and test coverage should be separated from C2. |
| `Mode` in `src/core/directions.rs` | reusable after refactoring | Basic variants match Julia, but `Mask(ArrayD<bool>)` owns the mask and cloning the mode deep-clones data. Prefer borrowed/generic mask input or a shared immutable owner. |
| `check_direction` | reusable after refactoring | Clear structured validation matches intended Julia rules, but it is stricter than actual Julia C2 because Julia C2 omits the check. |
| `C2Error` in `src/core/errors.rs` | reusable after refactoring | Useful messages, but it is C2-specific and encodes behavior (`LagHasNoTrials`) that differs from Julia's NaN result. A crate-wide error taxonomy belongs in M1. |
| `c2`/`c2_by` | useful as a naive/reference implementation after semantic repair | Direct coordinate traversal is clear and intentionally slow, but current mask and overlong-lag semantics are incompatible. `c2_by` is an unverified Rust extension rather than Julia parity. |
| `label_clusters` and neighbor traversal | reusable after refactoring; candidate independent oracle | Sequential face-connected BFS is understandable and periodic-aware. It needs independent tests, mask preprocessing by callers, and comparison against Julia/ImageMorphology before being trusted. |
| `advance`/`all_indices` | reusable after refactoring; candidate naive traversal | Logical multidimensional coordinates avoid flattened-order dependence. Mask target validity and Julia slice partition equivalence need tests. |
| inline C2 tests | preserve selectively and strengthen | The documented 1D case and boundary-wrapped component case are valuable. The mask test only covers an all-false mask and misses bridging/target normalization. Direction tests are sparse. |
| `src/main.rs` datasets and demo | reusable after refactoring as an example | Move to `examples/` in M1 if retained; it should not define the library architecture. |

No existing code is proven reusable as-is for parity. Nothing should be deleted in M0. The direct BFS and pair traversal are especially worth preserving as slow, transparent oracles once corrected and locked to Julia-derived fixtures.

## 6. Rust-vs-Julia C2 semantic comparison

### Shared behavior verified from source

- Both select a phase with equality before component labeling (`c2_by` is an additional Rust path).
- Both use face/Manhattan connectivity rather than diagonal connectivity.
- Both merge components across periodic faces.
- Output index 1 / Rust vector index 0 is zero lag, despite Julia prose describing distances from 1.
- Default length is half the minimum dimension.
- Non-periodic axial and diagonal pair traversal is intended to count every in-bounds directed origin-target pair once.
- Both return floating-point ratios (`Vector{Float64}` / `Vec<f64>`).

### Discrepancies and compatibility risks

Ten findings are tracked: seven source-confirmed behavioral/API differences and three risks requiring Julia execution or differential fixtures.

1. **Masked voxels participate in Rust components (confirmed, scientific).** Julia computes `masked = (array .== phase) .* mask` before labeling. Rust labels the unmasked phase indicator and consults the mask only when selecting origins. A masked voxel can therefore bridge two regions in Rust.
2. **Rust mask normalization does not require a valid target (confirmed, scientific).** Julia's mask autocorrelation counts only pairs where both origin and target masks are true. Rust increments `trials` for every in-bounds target of a mask-valid origin, without checking the target mask.
3. **No-valid-pair behavior differs outside mask mode (confirmed).** Julia division by zero yields `NaN`; Rust returns `LagHasNoTrials`, aborting the entire result.
4. **Overlong periodic lags differ (confirmed).** Julia updates numerator and denominator only through the slice length, leaving later entries as `0/0 -> NaN`. Rust continues modulo wrapping for every requested lag.
5. **Periodic non-cubic diagonals differ (confirmed).** Rust rejects them. Julia's general `check_direction` would reject them, but directional C2 never calls it; its periodic slicer runs using the first axis length. This is likely a Julia bug, but it is the pinned behavior and requires a deliberate compatibility decision.
6. **Rust exposes an extra `Direction::Diagonal` alias (confirmed API difference).** Julia has no generic diagonal designator; the alias maps to XY in 2D and XYZ in 3D and can hide dimensional intent.
7. **Rust exposes predicate-based `c2_by` (confirmed API extension).** Julia C2 supports equality against a phase value only. Julia surface docstrings mention predicate phases, but those implementations also use equality.
8. **Non-periodic 2D/3D labeling backend parity is not yet proven (risk).** Rust always uses its BFS; Julia delegates to the resolved ImageMorphology version. Intended connectivity matches, but label behavior must be differentially tested, including degenerate axes.
9. **Mask ownership/aliasing semantics differ (risk/API design).** Julia's `Mask` retains an array reference; Rust's owned ndarray and derived `Clone` deep-copy the mask. Values are equivalent if immutable, but mutation/large-mask behavior differs.
10. **Diagonal slice equivalence is not comprehensively tested (risk).** Source inspection indicates the Rust coordinate-step traversal matches Julia's partitions for normal non-periodic shapes and cubic periodic shapes, but `DirXZY` is absent from upstream tests and none of the Rust spatial diagonals except XYZ is tested.

Tiny explicit cases:

| Case | Julia C2 | Current Rust C2 | Cause |
| --- | --- | --- | --- |
| `array=[1,1,1]`, phase 1, mask `[true,false,true]`, non-periodic, lag 2 | `0/1 = 0` | `1/1 = 1` | masked center bridges endpoints during Rust labeling |
| `array=[1,0]`, phase 1, mask `[true,false]`, non-periodic, lag 1 | `0/0 = NaN` | `0/1 = 0` | Rust does not require target mask validity |
| `array=[1,1]`, phase 1, periodic, `len=3` | `[1,1,NaN]` | `[1,1,1]` | Rust wraps beyond one full slice; Julia stops updates |
| `array=[1,1]`, phase 1, non-periodic, `len=3` | `[1,1,NaN]` | error for lag with no trials | different invalid-normalization policy |
| 2x3 array, periodic `DirXY` | Julia computes first-axis-length wrapped slices | structured rejection | C2 bypasses Julia `check_direction` |

Phase filtering occurs before masking in Julia and Rust, but only Julia applies masking before connected components. In Julia masked origins and targets cannot match and mask autocorrelation defines the valid-pair denominator. In Rust only origins are mask-filtered. For unmasked modes, origin and target validity otherwise agree: origins range over every voxel; non-periodic targets must be in bounds; periodic targets wrap. Zero lag is phase volume fraction (or masked phase fraction), not identically one.

## 7. Code worth preserving

- Logical coordinate mappings in `direction_step`, after removing or explicitly documenting `Diagonal`.
- Structured validation and shape-aware errors, after separating Julia parity from Rust API ergonomics.
- Sequential face-connected BFS and periodic neighbor wrapping as an independently testable morphology primitive.
- Direct O(N^2)-style C2 pair traversal as a slow oracle, after correcting mask and invalid-normalization semantics.
- The documented 1D C2 test and periodic boundary-component test.
- Demonstration arrays from `main.rs`, moved to an example only if they remain useful.

## 8. Naive/reference implementation candidates

`label_clusters` plus direct coordinate traversal is the main candidate. It is simple, sequential, contains no unsafe code, and can remain independent of future FFT or optimized implementations. Before calling it an oracle, M2-M4/M8 must add exhaustive direction/slice tests, mask-pair normalization, Julia fixtures, and independent component-label tests. The current function is a prototype, not an oracle yet.

For future FFT work, add separate explicit naive autocorrelation and cross-correlation implementations rather than repurposing C2 traversal. For EDT, use brute-force Euclidean distances only in tests; do not replace the exact Julia algorithm during parity work.

## 9. Recommended target module structure

```text
src/
  lib.rs                  public API/re-exports
  error.rs                crate-wide validation/error types
  geometry/
    direction.rs          Julia direction set and rank validation
    slicing.rs            coordinate-exact directional slices
  boundary/
    mode.rs               periodic/non-periodic/mask representation
    normalization.rs      valid-pair counts and lag conventions
  transforms/
    directional.rs        sliced FFT plans/autocorrelation/cross-correlation
    map.rs                N-D map transforms and normalization
    naive.rs              small-array O(N^2) test oracles
  morphology/
    components.rs         face-connected labels, including periodic topology
    edt.rs                exact Julia-compatible EDT
    surface.rs            convolution/erosion edge extraction
  directional/
    s2.rs cc.rs l2.rs c2.rs pore_size.rs chord_length.rs
    surface2.rs three_point.rs surface3.rs
  map/
    s2.rs cc.rs c2.rs surface.rs directions.rs
examples/
  c2.rs                   optional former main demonstration
```

The exact filenames can change in M1, but the key boundaries should remain: geometry is independent of scientific functions; mask normalization is independent of phase data; connected components, EDT, and surfaces are separate morphology primitives; directional FFTs and N-D map FFTs are not conflated; naive oracles are retained.

## 10. Recommended migration order

The current milestone order remains sound:

1. M1: make a library crate and establish the module/error layout without changing scientific behavior; preserve the prototype C2 under an explicitly provisional/internal name.
2. M2: define exactly the 13 Julia directions, three boundary modes, rank rules, and mask ownership strategy.
3. M3: reproduce and exhaustively test slicer order for every rank/direction/mode, especially `DirXZY` and periodic diagonals.
4. M4: implement pair-validity/normalization independently, including both-end mask validity and zero-pair NaNs.
5. M5-M7: build naive-vs-FFT primitives, then S2/cross-correlation and L2.
6. M8: validate connected components independently, then repair/replace production C2 while keeping the slow oracle.
7. M9-M13: exact EDT, pore/chord, surface extraction/functions, then three-point functions.
8. M14 onward: maps, remaining utilities, and full parity audit.

## 11. Known documentation/API/test inconsistencies

The following should be treated as pinned-reference facts rather than silently "corrected" in Rust:

- Directional C2 omits `check_direction`, unlike S2, cross-correlation, and L2.
- `Map.c2` lacks its documented default mode.
- `DirXZY` is public and documented but omitted from `known_directions`.
- GPU tests use obsolete `periodic` keywords.
- The main example notebook also uses obsolete `periodic` keywords.
- Surface docstrings claim predicate phases; source performs equality.
- General dimensionality claims conflict with surface-kernel and map-direction implementations.
- `dir_from_map` documentation exceeds its direction methods and hard-coded index rank.
- Chord-length prose still mentions an old histogram/`nbins` interface.
- C2/S2/L2 prose says distances `1:len`, while first output is zero lag.
- Upstream has no manifest, so package source is pinned but dependency resolution is not exact.

## 12. Unresolved questions

1. Should Rust intentionally reproduce Julia C2's missing non-cubic-periodic validation, or document and test a deliberate safety difference? This requires project-owner approval after a Julia golden fixture demonstrates the pinned output.
2. What exact ImageMorphology version and structuring element are resolved in the intended reference environment? A manifest and direct component fixtures are needed.
3. Does `dir_from_map` actually pass the current 1D/2D upstream map tests under Julia 1.10, or is the pinned test suite stale/broken?
4. Are mask modes intentionally unsupported for all three-point and EDT/pore-size paths, or merely unimplemented?
5. Should the Rust API retain predicate phase selectors as an explicitly documented extension, or restrict the parity layer to equality?
6. For requested `len` beyond available lags, should the public Rust API reproduce Julia NaNs exactly or validate and reject while a compatibility API reproduces Julia? Scientific parity favors NaNs.
7. Which Julia registry snapshot should define exact dependency versions for differential testing?

## 13. M1 recommendations

M1 should be architecture-only. Convert the binary into a library with `lib.rs`, move the demonstration to `examples/`, separate boundary/direction/error/morphology concerns, and retain the existing C2 code as a clearly named provisional slow implementation. Do not repair C2 scientific semantics in M1.

Decisions M1 must make without implementing algorithms:

- represent masks without mandatory deep copies (generic borrow or shared immutable ownership);
- expose exactly the Julia direction set in the parity API and decide whether any convenience alias lives outside it;
- define a crate-wide error type while leaving numerical zero-pair behavior to M4;
- create module seams for independent directional and map transforms;
- keep component labeling, EDT, and surface extraction as separate morphology modules;
- provide test-support locations for Julia golden fixtures and naive implementations.

## Validation record

Commands run from the Rust repository root:

| Command | Result |
| --- | --- |
| `cargo fmt --check` | pass, exit 0 |
| `cargo test --all` | pass: 7 passed, 0 failed, 0 ignored |
| `cargo clippy --all-targets --all-features -- -D warnings` | fail: 5 pre-existing findings (one redundant closure and four upper-case-acronym enum variants) |
| `git -C reference/CorrelationFunctions.jl status --short` | clean |
| `julia --project=. -e "using Pkg; Pkg.test()"` in the reference | not run: `julia` command not found, exit 1 |

No tests were added and no Rust implementation file was modified in M0.
