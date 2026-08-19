use std::sync::Arc;

use ndarray::ArrayD;

use crate::error::{Error, Result};

/// A cheaply clonable, immutable mask handle.
///
/// Cloning this value clones an `Arc`, not the potentially large ndarray. The
/// array is exposed only through a shared reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    array: Arc<ArrayD<bool>>,
}

impl Mask {
    /// Transfer a mask array into shared immutable ownership.
    pub fn new(array: ArrayD<bool>) -> Self {
        Self {
            array: Arc::new(array),
        }
    }

    /// Borrow the mask array.
    pub fn array(&self) -> &ArrayD<bool> {
        &self.array
    }

    /// Return the logical ndarray shape used for input validation.
    pub fn shape(&self) -> &[usize] {
        self.array.shape()
    }
}

/// Boundary and mask mode for a scientific calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Wrap coordinates across every array boundary.
    Periodic,
    /// Do not wrap; later algorithms use only in-bounds data.
    NonPeriodic,
    /// Restrict valid data and pair counts to a boolean mask.
    ///
    /// Mask-mode boundary traversal follows non-periodic behavior in the
    /// pinned Julia two-point implementation. Pair normalization is deferred
    /// to M4.
    Mask(Mask),
}

impl Mode {
    /// Construct a mask mode and transfer the array into shared ownership.
    pub fn mask(mask: ArrayD<bool>) -> Self {
        Self::Mask(Mask::new(mask))
    }

    /// Return the mask, when this is a mask mode.
    ///
    /// The returned shared reference does not permit mutation:
    ///
    /// ```compile_fail
    /// use crf_rs::Mode;
    /// use ndarray::{ArrayD, IxDyn};
    ///
    /// let mode = Mode::mask(ArrayD::from_elem(IxDyn(&[2]), true));
    /// mode.mask_array().unwrap().fill(false);
    /// ```
    pub fn mask_array(&self) -> Option<&ArrayD<bool>> {
        match self {
            Self::Mask(mask) => Some(mask.array()),
            Self::Periodic | Self::NonPeriodic => None,
        }
    }

    /// Return the mask shape without exposing its ownership representation.
    pub fn mask_shape(&self) -> Option<&[usize]> {
        match self {
            Self::Mask(mask) => Some(mask.shape()),
            Self::Periodic | Self::NonPeriodic => None,
        }
    }
}

/// Validate that a mode's mask has exactly the input shape.
///
/// Julia checks this with an assertion inside `maybe_apply_mask`. Rust uses a
/// structured error as an intentional API-safety improvement; valid-input
/// scientific results are unaffected.
pub fn validate_mask_shape(mode: &Mode, input_shape: &[usize]) -> Result<()> {
    let Some(mask_shape) = mode.mask_shape() else {
        return Ok(());
    };

    if mask_shape == input_shape {
        Ok(())
    } else {
        Err(Error::MaskShapeMismatch {
            expected: input_shape.to_vec(),
            actual: mask_shape.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ndarray::{ArrayD, IxDyn};

    use super::{Mode, validate_mask_shape};
    use crate::error::Error;

    #[test]
    fn cloning_mask_mode_shares_the_mask_allocation() {
        let mode = Mode::mask(ArrayD::from_elem(IxDyn(&[2, 3]), true));
        let clone = mode.clone();

        let (Mode::Mask(mask), Mode::Mask(cloned_mask)) = (&mode, &clone) else {
            panic!("expected mask modes");
        };

        assert!(Arc::ptr_eq(&mask.array, &cloned_mask.array));
    }

    #[test]
    fn mask_shape_and_contents_are_available_immutably() {
        let mode = Mode::mask(
            ArrayD::from_shape_vec(IxDyn(&[2, 2]), vec![true, false, false, true]).unwrap(),
        );

        assert_eq!(mode.mask_shape(), Some([2, 2].as_slice()));
        assert_eq!(
            mode.mask_array()
                .unwrap()
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            [true, false, false, true]
        );
    }

    #[test]
    fn non_mask_modes_have_no_mask_storage() {
        for mode in [Mode::Periodic, Mode::NonPeriodic] {
            assert!(mode.mask_array().is_none());
            assert!(mode.mask_shape().is_none());
        }
    }

    #[test]
    fn exact_mask_shape_matches_are_accepted() {
        let mode = Mode::mask(ArrayD::from_elem(IxDyn(&[2, 3, 4]), true));
        assert!(validate_mask_shape(&mode, &[2, 3, 4]).is_ok());
    }

    #[test]
    fn one_axis_mask_shape_mismatches_are_diagnostic() {
        let mode = Mode::mask(ArrayD::from_elem(IxDyn(&[2, 3, 5]), true));
        assert_eq!(
            validate_mask_shape(&mode, &[2, 3, 4]),
            Err(Error::MaskShapeMismatch {
                expected: vec![2, 3, 4],
                actual: vec![2, 3, 5],
            })
        );
    }

    #[test]
    fn mask_rank_mismatches_are_diagnostic() {
        let mode = Mode::mask(ArrayD::from_elem(IxDyn(&[2, 3]), true));
        assert_eq!(
            validate_mask_shape(&mode, &[2, 3, 1]),
            Err(Error::MaskShapeMismatch {
                expected: vec![2, 3, 1],
                actual: vec![2, 3],
            })
        );
    }

    #[test]
    fn degenerate_and_empty_shapes_compare_exactly() {
        let degenerate = Mode::mask(ArrayD::from_elem(IxDyn(&[0, 3]), true));
        assert!(validate_mask_shape(&degenerate, &[0, 3]).is_ok());
        assert!(validate_mask_shape(&degenerate, &[3, 0]).is_err());

        let zero_rank = Mode::mask(ArrayD::from_elem(IxDyn(&[]), true));
        assert!(validate_mask_shape(&zero_rank, &[]).is_ok());
    }
}
