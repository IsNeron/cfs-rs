use ndarray::Array2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    X,
    Y,
    Z,
    Diagonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Periodic,
    NonPeriodic,
}

pub fn check_direction(
    direction: Direction,
    array: &Array2<f64>,
    mode: Mode,
) -> Result<Direction, String> {
    if !direction_predicate(array, direction) {
        return Err("Unknown directions found.".into());
    }

    let shape = array.shape();
    let cubic = shape.iter().all(|&x| x == shape[0]);
    let axial = matches!(direction, Direction::X | Direction::Y | Direction::Z);

    if mode == Mode::Periodic && !axial && !cubic {
        return Err("Periodic diagonals for non-cubic arrays are not supported".into());
    }

    Ok(direction)
}

fn direction_predicate(array: &Array2<f64>, direction: Direction) -> bool {
    let ndim = array.ndim();
    match (ndim, direction) {
        (1, Direction::X) => true,
        (2, Direction::X | Direction::Y | Direction::Diagonal) => true,
        (3, _) => true,
        _ => false,
    }
}
