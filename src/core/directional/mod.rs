use std::collections::VecDeque;

use ndarray::{Array, ArrayBase, ArrayD, Data, Dimension, IxDyn};

use crate::core::directions::{Direction, Mode, check_direction, default_len, direction_step};
use crate::core::errors::C2Error;

pub fn c2<T, S, D>(
    array: &ArrayBase<S, D>,
    phase: T,
    direction: Direction,
    mode: Mode,
    len: Option<usize>,
) -> Result<Vec<f64>, C2Error>
where
    S: Data<Elem = T>,
    D: Dimension,
    T: PartialEq,
{
    c2_by(array, |value| value == &phase, direction, mode, len)
}

pub fn c2_by<T, S, D, F>(
    array: &ArrayBase<S, D>,
    phase: F,
    direction: Direction,
    mode: Mode,
    len: Option<usize>,
) -> Result<Vec<f64>, C2Error>
where
    S: Data<Elem = T>,
    D: Dimension,
    F: Fn(&T) -> bool,
{
    let array = array.view().into_dyn();
    validate_mode(array.shape(), &mode)?;
    check_direction(direction, array.shape(), &mode)?;

    let len = len.unwrap_or_else(|| default_len(array.shape()));
    if len == 0 {
        return Ok(Vec::new());
    }

    let step = direction_step(direction, array.ndim())?;
    let coords = all_indices(&array);
    let mask = array.map(|value| phase(value));
    let labels = label_clusters(&mask, &mode);

    let mut result = Vec::with_capacity(len);
    for lag in 0..len {
        let mut matches = 0usize;
        let mut trials = 0usize;

        for coord in &coords {
            if !origin_allowed(coord, &mode) {
                continue;
            }

            if let Some(target) = advance(coord, &step, lag, array.shape(), &mode) {
                trials += 1;
                let lhs = labels[IxDyn(coord)];
                let rhs = labels[IxDyn(&target)];
                if lhs != 0 && lhs == rhs {
                    matches += 1;
                }
            }
        }

        push_statistic(&mut result, matches, trials, lag, &mode)?;
    }

    Ok(result)
}

fn validate_mode(shape: &[usize], mode: &Mode) -> Result<(), C2Error> {
    if let Mode::Mask(mask) = mode {
        let expected = shape.to_vec();
        let actual = mask.shape().to_vec();
        if expected != actual {
            return Err(C2Error::MaskShapeMismatch { expected, actual });
        }
    }
    Ok(())
}

fn push_statistic(
    result: &mut Vec<f64>,
    matches: usize,
    trials: usize,
    lag: usize,
    mode: &Mode,
) -> Result<(), C2Error> {
    if trials == 0 {
        if matches!(mode, Mode::Mask(_)) {
            result.push(f64::NAN);
            return Ok(());
        }
        return Err(C2Error::LagHasNoTrials { lag: lag + 1 });
    }

    result.push(matches as f64 / trials as f64);
    Ok(())
}

fn label_clusters(mask: &ArrayD<bool>, mode: &Mode) -> ArrayD<usize> {
    let mut labels = Array::zeros(mask.raw_dim());
    let mut next_label = 1usize;
    let coords = all_indices(mask);

    for coord in coords {
        let index = IxDyn(&coord);
        if !mask[index.clone()] || labels[index] != 0 {
            continue;
        }

        let mut queue = VecDeque::new();
        labels[IxDyn(&coord)] = next_label;
        queue.push_back(coord);

        while let Some(current) = queue.pop_front() {
            for neighbor in orthogonal_neighbors(&current, mask.shape(), mode) {
                let neighbor_ix = IxDyn(&neighbor);
                if mask[neighbor_ix.clone()] && labels[neighbor_ix] == 0 {
                    labels[IxDyn(&neighbor)] = next_label;
                    queue.push_back(neighbor);
                }
            }
        }

        next_label += 1;
    }

    labels
}

fn origin_allowed(coord: &[usize], mode: &Mode) -> bool {
    match mode {
        Mode::Mask(mask) => mask[IxDyn(coord)],
        Mode::Periodic | Mode::NonPeriodic => true,
    }
}

fn orthogonal_neighbors(coord: &[usize], shape: &[usize], mode: &Mode) -> Vec<Vec<usize>> {
    let mut neighbors = Vec::with_capacity(shape.len() * 2);

    for axis in 0..shape.len() {
        for delta in [-1isize, 1] {
            let mut next = coord.to_vec();
            let candidate = coord[axis] as isize + delta;

            match mode {
                Mode::Periodic => {
                    next[axis] = candidate.rem_euclid(shape[axis] as isize) as usize;
                    neighbors.push(next);
                }
                Mode::NonPeriodic | Mode::Mask(_) => {
                    if (0..shape[axis] as isize).contains(&candidate) {
                        next[axis] = candidate as usize;
                        neighbors.push(next);
                    }
                }
            }
        }
    }

    neighbors
}

fn advance(
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

fn all_indices<S>(array: &ArrayBase<S, IxDyn>) -> Vec<Vec<usize>>
where
    S: Data,
{
    array
        .indexed_iter()
        .map(|(index, _)| index.slice().to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use ndarray::{Array3, ArrayD, IxDyn, array};

    use super::{c2, c2_by};
    use crate::core::directions::{Direction, Mode};
    use crate::core::errors::C2Error;

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

        assert!(matches!(error, C2Error::PeriodicDiagonalRequiresCubicArray));
    }

    #[test]
    fn mask_mode_returns_nan_when_no_trials_are_available() {
        let array = array![1., 1., 1.];
        let mask = ArrayD::from_elem(IxDyn(&[3]), false);
        let result = c2(&array, 1.0, Direction::X, Mode::Mask(mask), Some(2)).unwrap();

        assert_close(&result, &[f64::NAN, f64::NAN]);
    }
}
