use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;

fn main() {
    let pwd = "test";
    
    // Values from encrypted_test.rar extra_block (I will hardcode testing)
    // Actually I can just write a quick script to parse it using rar::Archive
    let archive = rar::Archive::extract_all("tests/payloads/encrypted_test.rar", "/tmp/", "test").unwrap();
    let f = &archive.files[0];
    let feb = f.extra.file_encryption.as_ref().unwrap();
    
    let iter_number = 2u32.pow(feb.kdf_count.into());
    let mut key_extended = [0u8; 64];
    let _ = pbkdf2::<Hmac<Sha256>>(pwd.as_bytes(), &feb.salt, iter_number, &mut key_extended);
    
    println!("PW check bytes from RAR block: {:?}", feb.pw_check);
    println!("Derived bytes 32..44: {:?}", &key_extended[32..44]);
    println!("Derived bytes 0..12: {:?}", &key_extended[0..12]);
}
