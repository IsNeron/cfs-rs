//! Preserved direct C2 prototype.
//!
//! This implementation is intentionally provisional. The M0 audit documents
//! known differences from Julia involving masks, invalid normalization,
//! overlong lags, and periodic non-cubic diagonals. M1 relocates this code but
//! does not repair those semantics. It may become a slow reference
//! implementation after validation and repair in later milestones.

use ndarray::{ArrayBase, Data, Dimension, IxDyn};
use thiserror::Error as ThisError;

use crate::boundary::{Mode, validate_mask_shape};
use crate::error::Error;
use crate::geometry::slicing::{advance_coordinate, all_indices};
use crate::geometry::{Direction, default_len, validate_direction};
use crate::morphology::components::label_components_provisional;

/// Errors specific to the preserved, pre-parity C2 prototype.
///
/// Shared input validation uses the crate-wide [`Error`]. The no-trials
/// variant exists only to retain prototype behavior until M4 defines the
/// Julia-compatible zero-normalization policy.
#[derive(Debug, ThisError, PartialEq, Eq)]
pub enum ProvisionalC2Error {
    #[error(transparent)]
    Validation(#[from] Error),

    #[error("provisional C2 has no valid trials for lag {lag}")]
    LagHasNoTrials { lag: usize },
}

type ProvisionalC2Result<T> = std::result::Result<T, ProvisionalC2Error>;

/// Calculate C2 using the preserved direct prototype.
///
/// This function is exposed so the experimental code remains usable, but it
/// is not yet Julia-compatible. See `docs/reference_audit.md`.
pub fn c2<T, S, D>(
    array: &ArrayBase<S, D>,
    phase: T,
    direction: Direction,
    mode: Mode,
    len: Option<usize>,
) -> ProvisionalC2Result<Vec<f64>>
where
    S: Data<Elem = T>,
    D: Dimension,
    T: PartialEq,
{
    c2_by(array, |value| value == &phase, direction, mode, len)
}

/// Predicate variant of the preserved direct C2 prototype.
///
/// Predicate phase selection is a Rust prototype extension and is not part of
/// the audited Julia C2 API.
pub fn c2_by<T, S, D, F>(
    array: &ArrayBase<S, D>,
    phase: F,
    direction: Direction,
    mode: Mode,
    len: Option<usize>,
) -> ProvisionalC2Result<Vec<f64>>
where
    S: Data<Elem = T>,
    D: Dimension,
    F: Fn(&T) -> bool,
{
    let array = array.view().into_dyn();
    validate_mask_shape(&mode, array.shape())?;
    validate_direction(direction, array.shape(), &mode)?;

    let len = len.unwrap_or_else(|| default_len(array.shape()));
    if len == 0 {
        return Ok(Vec::new());
    }

    let step = direction.step_for_rank(array.ndim())?;
    let coords = all_indices(&array);
    let mask = array.map(phase);
    let labels = label_components_provisional(&mask, &mode);

    let mut result = Vec::with_capacity(len);
    for lag in 0..len {
        let mut matches = 0usize;
        let mut trials = 0usize;

        for coord in &coords {
            if !origin_allowed(coord, &mode) {
                continue;
            }

            if let Some(target) = advance_coordinate(coord, &step, lag, array.shape(), &mode) {
                trials += 1;
                let lhs = labels[IxDyn(coord)];
                let rhs = labels[IxDyn(&target)];
                if lhs != 0 && lhs == rhs {
                    matches += 1;
                }
            }
        }

        push_provisional_statistic(&mut result, matches, trials, lag, &mode)?;
    }

    Ok(result)
}

fn push_provisional_statistic(
    result: &mut Vec<f64>,
    matches: usize,
    trials: usize,
    lag: usize,
    mode: &Mode,
) -> ProvisionalC2Result<()> {
    if trials == 0 {
        if matches!(mode, Mode::Mask(_)) {
            result.push(f64::NAN);
            return Ok(());
        }
        return Err(ProvisionalC2Error::LagHasNoTrials { lag: lag + 1 });
    }

    result.push(matches as f64 / trials as f64);
    Ok(())
}

fn origin_allowed(coord: &[usize], mode: &Mode) -> bool {
    match mode {
        Mode::Mask(mask) => mask.array()[IxDyn(coord)],
        Mode::Periodic | Mode::NonPeriodic => true,
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{Array3, ArrayD, IxDyn, array};

    use super::{c2, c2_by};
    use crate::{Direction, Error, Mode, ProvisionalC2Error};

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (lhs, rhs) in actual.iter().zip(expected) {
            if lhs.is_nan() || rhs.is_nan() {
                assert!(lhs.is_nan() && rhs.is_nan());
            } else {
                assert!((lhs - rhs).abs() < 1e-9, "{lhs} != {rhs}");
            }
        }
    }

    #[test]
    fn c2_matches_documented_1d_example() {
        let array = array![1., 1., 1., 0., 1., 1.];
        let result = c2(&array, 1.0, Direction::X, Mode::NonPeriodic, Some(6)).unwrap();
        let expected = [5.0 / 6.0, 3.0 / 5.0, 1.0 / 4.0, 0.0, 0.0, 0.0];

        assert_close(&result, &expected);
    }

    #[test]
    fn c2_accepts_phase_predicate_like_julia_docs() {
        let array = array![0i32, 2, 4, 5, 6, 7];
        let result = c2_by(
            &array,
            |value| *value % 2 == 0,
            Direction::X,
            Mode::NonPeriodic,
            Some(3),
        )
        .unwrap();
        let expected = [4.0 / 6.0, 2.0 / 5.0, 1.0 / 4.0];

        assert_close(&result, &expected);
    }

    #[test]
    fn supports_negative_diagonal_in_2d() {
        let array = array![[0., 1.], [1., 0.]];
        let result = c2(&array, 1.0, Direction::YX, Mode::NonPeriodic, Some(2)).unwrap();
        let expected = [0.5, 0.0];

        assert_close(&result, &expected);
    }

    #[test]
    fn periodic_mode_wraps_clusters_across_boundaries() {
        let array = array![1., 0., 1., 1.];
        let result = c2(&array, 1.0, Direction::X, Mode::Periodic, Some(4)).unwrap();
        let expected = [0.75, 0.5, 0.5, 0.5];

        assert_close(&result, &expected);
    }

    #[test]
    fn supports_3d_diagonal_direction() {
        let array = Array3::from_elem((2, 2, 2), 1.0);
        let result = c2(&array, 1.0, Direction::XYZ, Mode::NonPeriodic, Some(2)).unwrap();
        let expected = [1.0, 1.0];

        assert_close(&result, &expected);
    }

    #[test]
    fn periodic_diagonal_requires_cubic_array() {
        let array = Array3::from_elem((2, 2, 3), 1.0);
        let error = c2(&array, 1.0, Direction::XYZ, Mode::Periodic, Some(1)).unwrap_err();

        assert!(matches!(
            error,
            ProvisionalC2Error::Validation(Error::PeriodicDirectionRequiresEqualAxes { .. })
        ));
    }

    #[test]
    fn mask_mode_returns_nan_when_no_trials_are_available() {
        let array = array![1., 1., 1.];
        let mask = ArrayD::from_elem(IxDyn(&[3]), false);
        let result = c2(&array, 1.0, Direction::X, Mode::mask(mask), Some(2)).unwrap();

        assert_close(&result, &[f64::NAN, f64::NAN]);
    }
}
