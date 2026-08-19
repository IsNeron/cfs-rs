//! Directional scientific functions.
//!
//! Only the preserved, provisional C2 prototype exists in M1.

#[path = "c2.rs"]
mod provisional_c2;

pub use provisional_c2::{ProvisionalC2Error, c2, c2_by};
