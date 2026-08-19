use crate::boundary::Mode;
use crate::error::{Error, Result};

/// A direction from the Julia-parity API.
///
/// Variant spelling deliberately follows the 13 exported Julia direction
/// designators. There is no generic `Diagonal` alias in the parity API.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Positive first axis: `[1]`, `[1, 0]`, or `[1, 0, 0]`.
    X,
    /// Positive second axis: `[0, 1]` or `[0, 1, 0]`.
    Y,
    /// Positive third axis: `[0, 0, 1]`.
    Z,
    /// Positive first and second axes: `[1, 1]` or `[1, 1, 0]`.
    XY,
    /// Negative first, positive second axis: `[-1, 1]` or `[-1, 1, 0]`.
    YX,
    /// Positive first and third axes: `[1, 0, 1]`.
    XZ,
    /// Negative first, positive third axis: `[-1, 0, 1]`.
    ZX,
    /// Positive second and third axes: `[0, 1, 1]`.
    YZ,
    /// Negative second, positive third axis: `[0, -1, 1]`.
    ZY,
    /// Positive first, second, and third axes: `[1, 1, 1]`.
    XYZ,
    /// Positive first, negative second, positive third axis: `[1, -1, 1]`.
    XZY,
    /// Negative first, positive second and third axes: `[-1, 1, 1]`.
    YXZ,
    /// Positive first and second, negative third axis: `[1, 1, -1]`.
    ZYX,
}

impl Direction {
    /// All 13 direction values exported by the pinned Julia reference.
    pub const ALL: [Self; 13] = [
        Self::X,
        Self::Y,
        Self::Z,
        Self::XY,
        Self::YX,
        Self::XZ,
        Self::ZX,
        Self::YZ,
        Self::ZY,
        Self::XYZ,
        Self::XZY,
        Self::YXZ,
        Self::ZYX,
    ];

    /// Return the direction's three-coordinate Julia step vector.
    ///
    /// Valid lower-rank representations use the leading `rank` coordinates.
    /// For example, X is `[1]` in 1D and `[1, 0]` in 2D. Call
    /// [`validate_direction_for_rank`] or [`Direction::step_for_rank`] before
    /// using a direction with a particular rank.
    pub const fn step(self) -> [isize; 3] {
        match self {
            Self::X => [1, 0, 0],
            Self::Y => [0, 1, 0],
            Self::Z => [0, 0, 1],
            Self::XY => [1, 1, 0],
            Self::YX => [-1, 1, 0],
            Self::XZ => [1, 0, 1],
            Self::ZX => [-1, 0, 1],
            Self::YZ => [0, 1, 1],
            Self::ZY => [0, -1, 1],
            Self::XYZ => [1, 1, 1],
            Self::XZY => [1, -1, 1],
            Self::YXZ => [-1, 1, 1],
            Self::ZYX => [1, 1, -1],
        }
    }

    /// Return the step vector at a rank for which this direction is valid.
    pub fn step_for_rank(self, rank: usize) -> Result<Vec<isize>> {
        validate_direction_for_rank(self, rank)?;
        Ok(self.step()[..rank].to_vec())
    }

    /// Whether this direction changes exactly one coordinate.
    pub const fn is_axial(self) -> bool {
        matches!(self, Self::X | Self::Y | Self::Z)
    }
}

/// Validate the directional API's supported rank range.
///
/// The pinned Julia `direction_predicate` errors with "Wrong number of
/// dimensions" outside 1D-3D. Rust exposes the same domain restriction as a
/// structured validation error.
pub fn validate_rank(rank: usize) -> Result<()> {
    if !(1..=3).contains(&rank) {
        return Err(Error::UnsupportedDimension { actual: rank });
    }

    Ok(())
}

/// Validate that a Julia-parity direction exists for the given rank.
pub fn validate_direction_for_rank(direction: Direction, rank: usize) -> Result<()> {
    validate_rank(rank)?;

    let valid = match rank {
        1 => matches!(direction, Direction::X),
        2 => matches!(
            direction,
            Direction::X | Direction::Y | Direction::XY | Direction::YX
        ),
        3 => true,
        _ => unreachable!("rank was validated above"),
    };

    if valid {
        Ok(())
    } else {
        Err(Error::InvalidDirectionForRank { direction, rank })
    }
}

/// Validate shape constraints imposed by a boundary mode and direction.
///
/// Julia's shared `check_direction` requires all axes to have equal lengths
/// for every periodic non-axial direction. It does not constrain axial,
/// non-periodic, or mask-mode shapes.
pub fn validate_direction_shape(direction: Direction, shape: &[usize], mode: &Mode) -> Result<()> {
    validate_rank(shape.len())?;

    let equal_axes = shape.iter().all(|&axis| axis == shape[0]);
    if matches!(mode, Mode::Periodic) && !direction.is_axial() && !equal_axes {
        return Err(Error::PeriodicDirectionRequiresEqualAxes {
            direction,
            shape: shape.to_vec(),
        });
    }

    Ok(())
}

/// Perform all shared Julia direction checks in their composable order.
pub fn validate_direction(direction: Direction, shape: &[usize], mode: &Mode) -> Result<()> {
    validate_rank(shape.len())?;
    validate_direction_for_rank(direction, shape.len())?;
    validate_direction_shape(direction, shape, mode)
}

pub(crate) fn default_len(shape: &[usize]) -> usize {
    shape.iter().copied().min().unwrap_or(0) / 2
}

#[cfg(test)]
mod tests {
    use super::{
        Direction, validate_direction, validate_direction_for_rank, validate_direction_shape,
        validate_rank,
    };
    use crate::boundary::Mode;
    use crate::error::Error;

    const STEPS: [(Direction, [isize; 3]); 13] = [
        (Direction::X, [1, 0, 0]),
        (Direction::Y, [0, 1, 0]),
        (Direction::Z, [0, 0, 1]),
        (Direction::XY, [1, 1, 0]),
        (Direction::YX, [-1, 1, 0]),
        (Direction::XZ, [1, 0, 1]),
        (Direction::ZX, [-1, 0, 1]),
        (Direction::YZ, [0, 1, 1]),
        (Direction::ZY, [0, -1, 1]),
        (Direction::XYZ, [1, 1, 1]),
        (Direction::XZY, [1, -1, 1]),
        (Direction::YXZ, [-1, 1, 1]),
        (Direction::ZYX, [1, 1, -1]),
    ];

    #[test]
    fn all_direction_steps_match_the_pinned_julia_definitions() {
        assert_eq!(Direction::ALL, STEPS.map(|(direction, _)| direction));

        for (direction, expected) in STEPS {
            assert_eq!(direction.step(), expected, "wrong step for {direction:?}");
        }

        assert_eq!(Direction::XZY.step(), [1, -1, 1]);
    }

    #[test]
    fn lower_rank_steps_are_julia_vector_prefixes() {
        assert_eq!(Direction::X.step_for_rank(1).unwrap(), [1]);
        assert_eq!(Direction::X.step_for_rank(2).unwrap(), [1, 0]);
        assert_eq!(Direction::XY.step_for_rank(2).unwrap(), [1, 1]);
        assert_eq!(Direction::YX.step_for_rank(2).unwrap(), [-1, 1]);
        assert_eq!(Direction::ZYX.step_for_rank(3).unwrap(), [1, 1, -1]);
    }

    #[test]
    fn every_direction_rank_pair_matches_julia_validity() {
        for rank in 1..=3 {
            for direction in Direction::ALL {
                let expected = match rank {
                    1 => direction == Direction::X,
                    2 => matches!(
                        direction,
                        Direction::X | Direction::Y | Direction::XY | Direction::YX
                    ),
                    3 => true,
                    _ => unreachable!(),
                };

                assert_eq!(
                    validate_direction_for_rank(direction, rank).is_ok(),
                    expected,
                    "unexpected validity for {direction:?} at rank {rank}"
                );
            }
        }
    }

    #[test]
    fn ranks_outside_one_through_three_are_rejected() {
        for rank in [0, 4, 8] {
            assert_eq!(
                validate_rank(rank),
                Err(Error::UnsupportedDimension { actual: rank })
            );
        }
    }

    #[test]
    fn periodic_2d_shape_rules_match_julia_shared_validation() {
        for direction in [Direction::X, Direction::Y] {
            assert!(validate_direction(direction, &[3, 5], &Mode::Periodic).is_ok());
        }

        for direction in [Direction::XY, Direction::YX] {
            assert!(validate_direction(direction, &[4, 4], &Mode::Periodic).is_ok());
            assert_eq!(
                validate_direction(direction, &[3, 5], &Mode::Periodic),
                Err(Error::PeriodicDirectionRequiresEqualAxes {
                    direction,
                    shape: vec![3, 5],
                })
            );
        }
    }

    #[test]
    fn periodic_3d_shape_rules_match_julia_shared_validation() {
        let non_cubic = [4, 4, 5];

        for direction in [Direction::X, Direction::Y, Direction::Z] {
            assert!(validate_direction(direction, &non_cubic, &Mode::Periodic).is_ok());
        }

        for direction in [
            Direction::XY,
            Direction::YX,
            Direction::XZ,
            Direction::ZX,
            Direction::YZ,
            Direction::ZY,
        ] {
            assert_eq!(
                validate_direction(direction, &non_cubic, &Mode::Periodic),
                Err(Error::PeriodicDirectionRequiresEqualAxes {
                    direction,
                    shape: non_cubic.to_vec(),
                }),
                "Julia requires all three axes to match even for planar {direction:?}"
            );
        }

        for direction in [
            Direction::XYZ,
            Direction::XZY,
            Direction::YXZ,
            Direction::ZYX,
        ] {
            assert!(validate_direction(direction, &[4, 4, 4], &Mode::Periodic).is_ok());
            assert!(validate_direction(direction, &non_cubic, &Mode::Periodic).is_err());
        }
    }

    #[test]
    fn non_periodic_and_mask_modes_do_not_require_equal_axes() {
        for direction in Direction::ALL {
            assert!(validate_direction_shape(direction, &[2, 3, 4], &Mode::NonPeriodic).is_ok());
        }

        let mask = ndarray::ArrayD::from_elem(ndarray::IxDyn(&[2, 3, 4]), true);
        assert!(validate_direction_shape(Direction::XYZ, &[2, 3, 4], &Mode::mask(mask)).is_ok());
    }
}
