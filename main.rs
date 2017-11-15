use std::io;

fn main() {
    println!("Enter the file paths, comma-separated:");
    let mut files = String::new();
    io::stdin().read_line(&mut files)
        .expect("Failed to read file paths.");
    let files = files.trim().split(",");
}