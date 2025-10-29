mod core;

use core::directions::{Direction, Mode, check_direction};

use ndarray::Array2;

fn main() {
    let array = Array2::from_shape_vec((3, 4), (1..=12).map(|x| x as f64).collect()).unwrap();
    match check_direction(Direction::X, &array, Mode::Periodic) {
        Ok(dir) => println!("✅ {:?} ok", dir),
        Err(e) => eprintln!("❌ Ошибка: {e}"),
    }
    println!("Array:\n{array}");
}
