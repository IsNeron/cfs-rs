use std::collections::VecDeque;

use ndarray::{Array, ArrayBase, ArrayD, Data, Dimension, IxDyn};

use crate::core::directions::{Direction, Mode, check_direction, default_len, direction_step};

pub fn c2<S, D>(
    array: &ArrayBase<S, D>,
    phase: f64,
    direction: Direction,
    mode: Mode,
    len: Option<usize>,
) -> Result<Vec<f64>, String>
where
    S: Data<Elem = f64>,
    D: Dimension,
{
    let array = array.view().into_dyn();
    check_direction(direction, array.shape(), mode)?;

    let len = len.unwrap_or_else(|| default_len(array.shape()));
    if len == 0 {
        return Ok(Vec::new());
    }

    let step = direction_step(direction, array.ndim())?;
    let coords = all_indices(&array);
    let mask = array.mapv(|value| value == phase);
    let labels = label_clusters(&mask, mode);

    let mut result = Vec::with_capacity(len);
    for lag in 0..len {
        let mut matches = 0usize;
        let mut trials = 0usize;

        for coord in &coords {
            if let Some(target) = advance(coord, &step, lag, array.shape(), mode) {
                trials += 1;
                let lhs = labels[IxDyn(coord)];
                let rhs = labels[IxDyn(&target)];
                if lhs != 0 && lhs == rhs {
                    matches += 1;
                }
            }
        }

        if trials == 0 {
            return Err(format!(
                "Lag {} is outside the selected array and direction.",
                lag + 1
            ));
        }
        result.push(matches as f64 / trials as f64);
    }

    Ok(result)
}

fn label_clusters(mask: &ArrayD<bool>, mode: Mode) -> ArrayD<usize> {
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

fn orthogonal_neighbors(coord: &[usize], shape: &[usize], mode: Mode) -> Vec<Vec<usize>> {
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
                Mode::NonPeriodic => {
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
    mode: Mode,
) -> Option<Vec<usize>> {
    let mut next = Vec::with_capacity(coord.len());

    for axis in 0..coord.len() {
        let candidate = coord[axis] as isize + step[axis] * lag as isize;
        match mode {
            Mode::Periodic => {
                next.push(candidate.rem_euclid(shape[axis] as isize) as usize);
            }
            Mode::NonPeriodic => {
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
    use ndarray::{Array3, array};

    use super::c2;
    use crate::core::directions::{Direction, Mode};

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (lhs, rhs) in actual.iter().zip(expected) {
            assert!((lhs - rhs).abs() < 1e-9, "{lhs} != {rhs}");
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

        assert!(error.contains("Periodic diagonals"));
    }
}
