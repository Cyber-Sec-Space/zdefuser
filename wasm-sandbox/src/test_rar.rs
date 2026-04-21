use rar::Archive;

fn main() {
    let mut archive = Archive::extract_all("dummy_file", "dummy_path", "dummy_password").unwrap();
    let num_files = archive.files.len();
    for file_block in archive.files.iter() {
        let name = file_block.name.clone();
    }
}
