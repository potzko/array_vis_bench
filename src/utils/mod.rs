pub mod array_gen;
pub mod small_sort;
pub mod check_utils;
pub mod rotation;
pub mod shell_branching;
pub mod shell_sequences;

use std::io::stdin;
pub fn read_num_stdin() -> usize {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("failed to read");
    buffer.trim().parse().unwrap_or(0)
}
