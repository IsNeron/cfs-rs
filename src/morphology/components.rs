//! Provisional face-connected component labeling.
//!
//! This is the BFS implementation preserved from the prototype. It has not
//! yet been validated against Julia/ImageMorphology and must not be described
//! as Julia-compatible. Validation and semantic repair belong to M8.

use std::collections::VecDeque;

use ndarray::{Array, ArrayD, IxDyn};

use crate::boundary::Mode;
use crate::geometry::slicing::all_indices;

pub(crate) fn label_components_provisional(mask: &ArrayD<bool>, mode: &Mode) -> ArrayD<usize> {
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
