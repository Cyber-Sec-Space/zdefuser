use std::fs::File;

fn main() {
    let pwd = "test";
    let file = File::open("tests/payloads/encrypted_test.rar").unwrap();
    let mut reader = rar::RarReader::new_from_file(file);
    // well rar crate API isn't exactly public for internal parsing, let's just use it
}
