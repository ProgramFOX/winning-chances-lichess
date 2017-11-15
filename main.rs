use std::io;

extern crate winning_chances_core;
extern crate regex;

use winning_chances_core::calculate_from_files;

fn main() {
    println!("Enter the file paths, comma-separated:");
    let mut files = String::new();
    io::stdin().read_line(&mut files)
        .expect("Failed to read file paths.");
    let files = files.trim().split(",");
    println!("{}", calculate_from_files(files));
}