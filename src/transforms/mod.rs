//! Correlation transform architecture.
//!
//! Directional sliced transforms and N-dimensional map transforms remain
//! separate. No FFT functionality is implemented in M1.

pub mod directional;
pub mod map;
pub(crate) mod naive;
