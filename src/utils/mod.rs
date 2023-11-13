pub mod array_gen;
pub mod check_utils;

use std::io::stdin;
pub fn read_num_stdin() -> usize {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("failed to read");
    buffer.trim().parse().unwrap_or(100)
}
