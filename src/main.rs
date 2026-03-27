mod core;

use core::directional::c2;
use core::directions::{Direction, Mode};

use ndarray::{Array3, array};

fn main() {
    let line = array![1., 1., 1., 0., 1., 1.];
    let porous_channel = array![
        [[1., 1., 0.], [1., 0., 0.], [1., 1., 0.]],
        [[1., 1., 0.], [1., 0., 0.], [1., 1., 0.]],
        [[1., 1., 0.], [1., 1., 0.], [0., 0., 0.]],
    ];
    let porous_islands = array![
        [[1., 0., 1.], [0., 0., 0.], [1., 0., 1.]],
        [[0., 0., 0.], [0., 1., 0.], [0., 0., 0.]],
        [[1., 0., 1.], [0., 0., 0.], [1., 0., 1.]],
    ];
    let layered_porous_medium = array![
        [
            [1., 1., 1., 0.],
            [1., 0., 1., 0.],
            [1., 0., 1., 0.],
            [1., 1., 1., 0.]
        ],
        [
            [1., 1., 1., 0.],
            [0., 0., 1., 0.],
            [1., 1., 1., 0.],
            [0., 0., 0., 0.]
        ],
        [
            [1., 0., 0., 0.],
            [1., 1., 1., 0.],
            [0., 1., 0., 0.],
            [0., 1., 1., 1.]
        ],
        [
            [1., 0., 0., 0.],
            [1., 1., 1., 0.],
            [0., 0., 1., 0.],
            [0., 1., 1., 1.]
        ],
    ];

    match c2(&line, 1.0, Direction::X, Mode::NonPeriodic, Some(6)) {
        Ok(values) => println!("1D C2 along X: {values:?}"),
        Err(error) => eprintln!("C2 calculation failed: {error}"),
    }

    println!("\n3D porous media examples:");
    run_3d_example(
        "Porous channel",
        &porous_channel,
        &[
            (Direction::X, Mode::NonPeriodic, 3),
            (Direction::Z, Mode::NonPeriodic, 3),
            (Direction::XYZ, Mode::NonPeriodic, 2),
        ],
    );
    run_3d_example(
        "Porous islands",
        &porous_islands,
        &[
            (Direction::X, Mode::NonPeriodic, 3),
            (Direction::XY, Mode::NonPeriodic, 2),
            (Direction::XYZ, Mode::NonPeriodic, 2),
        ],
    );
    run_3d_example(
        "Layered porous medium",
        &layered_porous_medium,
        &[
            (Direction::Y, Mode::NonPeriodic, 4),
            (Direction::XZ, Mode::NonPeriodic, 3),
            (Direction::XYZ, Mode::Periodic, 2),
        ],
    );
}

fn run_3d_example(name: &str, sample: &Array3<f64>, configs: &[(Direction, Mode, usize)]) {
    println!("\n{name} (shape {:?}):\n{sample}", sample.dim());

    for (direction, mode, len) in configs {
        match c2(sample, 1.0, *direction, mode.clone(), Some(*len)) {
            Ok(values) => println!("  {direction:?} {mode:?}: {values:?}"),
            Err(error) => println!("  {direction:?} {mode:?}: error: {error}"),
        }
    }
}
