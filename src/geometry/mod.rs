//! Array geometry independent of individual scientific functions.

mod direction;
pub(crate) mod slicing;

pub(crate) use direction::default_len;
pub use direction::{
    Direction, validate_direction, validate_direction_for_rank, validate_direction_shape,
    validate_rank,
};
