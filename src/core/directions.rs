use ndarray::ArrayD;

use crate::core::errors::C2Error;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    X,
    Y,
    Z,
    XY,
    YX,
    XZ,
    ZX,
    YZ,
    ZY,
    XYZ,
    XZY,
    YXZ,
    ZYX,
    Diagonal,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Periodic,
    NonPeriodic,
    Mask(ArrayD<bool>),
}

pub fn check_direction(direction: Direction, shape: &[usize], mode: &Mode) -> Result<(), C2Error> {
    let ndim = shape.len();
    if !(1..=3).contains(&ndim) {
        return Err(C2Error::UnsupportedDimension(ndim));
    }

    direction_step(direction, ndim)?;

    let cubic = shape.iter().all(|&axis| axis == shape[0]);
    let axial = matches!(direction, Direction::X | Direction::Y | Direction::Z);
    if matches!(mode, Mode::Periodic) && !axial && !cubic {
        return Err(C2Error::PeriodicDiagonalRequiresCubicArray);
    }

    Ok(())
}

pub fn direction_step(direction: Direction, ndim: usize) -> Result<Vec<isize>, C2Error> {
    match (ndim, direction) {
        (1, Direction::X) => Ok(vec![1]),
        (2, Direction::X) => Ok(vec![1, 0]),
        (2, Direction::Y) => Ok(vec![0, 1]),
        (2, Direction::XY | Direction::Diagonal) => Ok(vec![1, 1]),
        (2, Direction::YX) => Ok(vec![-1, 1]),
        (3, Direction::X) => Ok(vec![1, 0, 0]),
        (3, Direction::Y) => Ok(vec![0, 1, 0]),
        (3, Direction::Z) => Ok(vec![0, 0, 1]),
        (3, Direction::XY) => Ok(vec![1, 1, 0]),
        (3, Direction::YX) => Ok(vec![-1, 1, 0]),
        (3, Direction::XZ) => Ok(vec![1, 0, 1]),
        (3, Direction::ZX) => Ok(vec![-1, 0, 1]),
        (3, Direction::YZ) => Ok(vec![0, 1, 1]),
        (3, Direction::ZY) => Ok(vec![0, -1, 1]),
        (3, Direction::XYZ | Direction::Diagonal) => Ok(vec![1, 1, 1]),
        (3, Direction::XZY) => Ok(vec![1, -1, 1]),
        (3, Direction::YXZ) => Ok(vec![-1, 1, 1]),
        (3, Direction::ZYX) => Ok(vec![1, 1, -1]),
        _ => Err(C2Error::UnsupportedDirection { direction, ndim }),
    }
}

pub fn default_len(shape: &[usize]) -> usize {
    shape.iter().copied().min().unwrap_or(0) / 2
}
