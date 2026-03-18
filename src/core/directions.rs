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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Periodic,
    NonPeriodic,
}

pub fn check_direction(direction: Direction, shape: &[usize], mode: Mode) -> Result<(), String> {
    let ndim = shape.len();
    if !(1..=3).contains(&ndim) {
        return Err(format!(
            "C2 currently supports only 1D, 2D or 3D arrays, got {ndim}D."
        ));
    }

    direction_step(direction, ndim)?;

    let cubic = shape.iter().all(|&axis| axis == shape[0]);
    let axial = matches!(direction, Direction::X | Direction::Y | Direction::Z);
    if mode == Mode::Periodic && !axial && !cubic {
        return Err("Periodic diagonals for non-cubic arrays are not supported".into());
    }

    Ok(())
}

pub fn direction_step(direction: Direction, ndim: usize) -> Result<Vec<isize>, String> {
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
        _ => Err(format!(
            "Direction {:?} is not supported for {ndim}D arrays.",
            direction
        )),
    }
}

pub fn default_len(shape: &[usize]) -> usize {
    shape.iter().copied().min().unwrap_or(0) / 2
}
