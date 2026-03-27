use crate::core::directions::Direction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum C2Error {
    #[error("C2 currently supports only 1D, 2D or 3D arrays, got {0}D.")]
    UnsupportedDimension(usize),
    #[error("direction {direction:?} is not supported for {ndim}D arrays")]
    UnsupportedDirection { direction: Direction, ndim: usize },
    #[error("periodic diagonals for non-cubic arrays are not supported")]
    PeriodicDiagonalRequiresCubicArray,
    #[error("mask shape {actual:?} does not match input array shape {expected:?}")]
    MaskShapeMismatch {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
    #[error("lag {lag} has no valid trials for the selected direction and boundary mode")]
    LagHasNoTrials { lag: usize },
}
