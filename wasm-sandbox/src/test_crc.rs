fn main() {
    let archive = rar::Archive::extract_all("tests/payloads/encrypted_test.rar", "/tmp/", "test").unwrap();
    let f = &archive.files[0];
    println!("crc: {}", f.data_crc);
}
