use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;

fn main() {
    let pwd = "test";
    let kdf_count = 15; // typically 15 means 2^15 = 32768
    // I need the salt from encrypted_test.rar
    // Let me just read it using the rar crate
}
