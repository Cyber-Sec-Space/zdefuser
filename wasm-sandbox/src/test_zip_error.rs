use zip::result::ZipError;

fn main() {
    let err = ZipError::UnsupportedArchive("Password required to decrypt file");
    match err {
        ZipError::UnsupportedArchive(msg) if msg.contains("Password required") => println!("Matched msg: {}", msg),
        e => println!("Other: {}", e),
    }
}
