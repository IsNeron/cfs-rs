mod core;

use core::autocorr::c2;
use core::directions::{Direction, Mode};

use ndarray::{Array3, array};

fn main() {
    let line = array![1., 1., 1., 0., 1., 1.];
    let cube = Array3::from_elem((2, 2, 2), 1.0);

    match c2(&line, 1.0, Direction::X, Mode::NonPeriodic, Some(6)) {
        Ok(values) => println!("1D C2 along X: {values:?}"),
        Err(error) => eprintln!("C2 calculation failed: {error}"),
    }

    match c2(&cube, 1.0, Direction::XYZ, Mode::NonPeriodic, Some(2)) {
        Ok(values) => println!("3D C2 along XYZ: {values:?}"),
        Err(error) => eprintln!("C2 calculation failed: {error}"),
    }
}
