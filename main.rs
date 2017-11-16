use std::io;

extern crate time;
extern crate winning_chances_core;

use winning_chances_core::calculate_from_files;

fn main() {
    let start = time::PreciseTime::now();
    println!("Enter the file paths, comma-separated:");
    let mut files = String::new();
    io::stdin()
        .read_line(&mut files)
        .expect("Failed to read file paths.");
    let files = files.trim().split(",");
    calculate_from_files(files);
    let duration = start.to(time::PreciseTime::now());
    println!("Time: {} seconds", duration.num_seconds());
}
