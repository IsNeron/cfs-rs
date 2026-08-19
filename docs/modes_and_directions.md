# Modes and directions (M2)

This document records the shared direction and boundary-mode contracts taken
from the pinned Julia reference at commit
`c76f1625368ae4a1c6326b9055e51ddcc9bcf19e`. It establishes only M2 parity;
directional slicing and normalization are not implemented here.

## Direction vectors

| Rust | Julia | 3-coordinate step | Valid ranks |
| --- | --- | --- | --- |
| `X` | `DirX` | `[1, 0, 0]` | 1, 2, 3 |
| `Y` | `DirY` | `[0, 1, 0]` | 2, 3 |
| `Z` | `DirZ` | `[0, 0, 1]` | 3 |
| `XY` | `DirXY` | `[1, 1, 0]` | 2, 3 |
| `YX` | `DirYX` | `[-1, 1, 0]` | 2, 3 |
| `XZ` | `DirXZ` | `[1, 0, 1]` | 3 |
| `ZX` | `DirZX` | `[-1, 0, 1]` | 3 |
| `YZ` | `DirYZ` | `[0, 1, 1]` | 3 |
| `ZY` | `DirZY` | `[0, -1, 1]` | 3 |
| `XYZ` | `DirXYZ` | `[1, 1, 1]` | 3 |
| `XZY` | `DirXZY` | `[1, -1, 1]` | 3 |
| `YXZ` | `DirYXZ` | `[-1, 1, 1]` | 3 |
| `ZYX` | `DirZYX` | `[1, 1, -1]` | 3 |

For valid 1D and 2D uses, the step is the corresponding leading prefix. The
parity API contains no generic `Diagonal` direction.

Julia's `direction_predicate` accepts only X in 1D, X/Y/XY/YX in 2D, and all
13 directions in 3D. It throws `error("Wrong number of dimensions")` for any
other rank. Rust represents that condition with `Error::UnsupportedDimension`.

## Periodic shape rule

Julia's shared `check_direction` permits arbitrary shapes for axial
directions and for every non-periodic or mask-mode direction. For a periodic
non-axial direction, **all array axes must have the same length**. In 3D this
means a planar XY direction still requires the Z length to match X and Y.

Directional C2 in the pinned Julia source does not call `check_direction` and
therefore accidentally bypasses this rule. The Rust shared geometry validator
implements the intended Julia rule. The preserved Rust C2 prototype continues
to reject periodic non-cubic diagonals as it did before M2; the final
C2-specific compatibility decision remains deferred to M8.

## Boundary modes and masks

- `Periodic` means coordinates wrap across array boundaries.
- `NonPeriodic` means later algorithms use only in-bounds data.
- `Mask` restricts valid samples and pair normalization to a boolean ndarray;
  in Julia's two-point paths its boundary traversal otherwise behaves as
  non-periodic.

Rust owns a mask through a private `Arc<ArrayD<bool>>`. Cloning a mask or mode
shares the allocation, and the public API exposes only immutable array and
shape references. Periodic and non-periodic modes contain no mask allocation.

Julia's `maybe_apply_mask` checks shape equality with an assertion. Rust uses
the composable `validate_mask_shape` function and returns the structured
`Error::MaskShapeMismatch`. This is an intentional API-safety difference that
does not change results for valid inputs.
