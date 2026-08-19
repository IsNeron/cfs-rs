//! Shared validation errors for the crate.

use thiserror::Error;

use crate::geometry::Direction;

/// Errors raised while validating correlation-function inputs.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// The input rank is outside the currently supported range.
    #[error("only 1D, 2D, and 3D arrays are currently supported, got {actual}D")]
    UnsupportedDimension { actual: usize },

    /// A direction is not defined for the input rank.
    #[error("direction {direction:?} is not supported for {rank}D arrays")]
    InvalidDirectionForRank { direction: Direction, rank: usize },

    /// A periodic non-axial direction was requested for unequal axis lengths.
    #[error("periodic direction {direction:?} requires equal axis lengths, got shape {shape:?}")]
    PeriodicDirectionRequiresEqualAxes {
        direction: Direction,
        shape: Vec<usize>,
    },

    /// A mask and its associated input have different shapes.
    #[error("mask shape {actual:?} does not match input array shape {expected:?}")]
    MaskShapeMismatch {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
}

/// Result type used by crate APIs.
pub type Result<T> = std::result::Result<T, Error>;
