//! Boundary conditions and masks shared by scientific functions.
//!
//! Pair-count normalization is intentionally deferred to M4.

mod mode;
pub mod normalization;

pub use mode::{Mask, Mode, validate_mask_shape};
