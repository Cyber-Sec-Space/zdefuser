use std::fs::File;
use std::path::Path;

fn main() {
    let f = File::open("tests/payloads/01_path_traversal.tar").unwrap();
    let mut archive = rar::Archive::new(f);
}
