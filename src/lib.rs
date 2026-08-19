//! Rust migration of `CorrelationFunctions.jl`.
//!
//! The crate is under active parity development. The currently exposed C2
//! implementation is a preserved prototype and is not yet Julia-compatible.

pub mod boundary;
pub mod directional;
pub mod error;
pub mod geometry;
pub mod map;
pub mod morphology;
pub mod transforms;

pub use boundary::{Mask, Mode, validate_mask_shape};
pub use directional::{ProvisionalC2Error, c2, c2_by};
pub use error::{Error, Result};
pub use geometry::{
    Direction, validate_direction, validate_direction_for_rank, validate_direction_shape,
    validate_rank,
};
