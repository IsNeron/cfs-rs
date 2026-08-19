//! Directional slicing seam.
//!
//! The complete Julia-compatible slicer belongs to M3. The coordinate helpers
//! below are preserved mechanically for the provisional C2 implementation.

use ndarray::{ArrayBase, Data, Dimension, IxDyn};

use crate::boundary::Mode;

pub(crate) fn advance_coordinate(
    coord: &[usize],
    step: &[isize],
    lag: usize,
    shape: &[usize],
    mode: &Mode,
) -> Option<Vec<usize>> {
    let mut next = Vec::with_capacity(coord.len());

    for axis in 0..coord.len() {
        let candidate = coord[axis] as isize + step[axis] * lag as isize;
        match mode {
            Mode::Periodic => {
                next.push(candidate.rem_euclid(shape[axis] as isize) as usize);
            }
            Mode::NonPeriodic | Mode::Mask(_) => {
                if !(0..shape[axis] as isize).contains(&candidate) {
                    return None;
                }
                next.push(candidate as usize);
            }
        }
    }

    Some(next)
}

pub(crate) fn all_indices<S>(array: &ArrayBase<S, IxDyn>) -> Vec<Vec<usize>>
where
    S: Data,
{
    array
        .indexed_iter()
        .map(|(index, _)| index.slice().to_vec())
        .collect()
}
