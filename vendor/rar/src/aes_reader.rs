use aes::Aes256;
#[allow(deprecated)]
use cbc::cipher::generic_array::GenericArray;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;
use std::io::{Read, Result, Seek, SeekFrom};

use crate::extra_block::FileEncryptionBlock;
use crate::file_block::FileBlock;

type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// RAR Decryption reader to decrypt .rar archive files
pub struct RarAesReader<R: Read> {
    /// Reader to read encrypted data from
    reader: R,
    /// Define if the encryption is active
    active: bool,
    /// Decrypted buffer
    buffer: Vec<u8>,
    /// Position in buffer
    buffer_pos: usize,
    /// Decryptor instance
    decryptor: Option<Aes256CbcDec>,
    /// Buffer for accumulating encrypted data
    encrypted_buffer: Vec<u8>,
}

impl<R: Read> RarAesReader<R> {
    /// Create a new decryption reader
    pub fn new(reader: R, file: FileBlock, pwd: &str) -> RarAesReader<R> {
        let mut active = false;
        let mut decryptor = None;

        if let Some(f) = file.extra.file_encryption {
            let key = generate_key(&f, pwd);
            decryptor = Some(Aes256CbcDec::new(&key.into(), &f.init.into()));
            active = true;
        }

        RarAesReader {
            reader,
            active,
            buffer: Vec::new(),
            buffer_pos: 0,
            decryptor,
            encrypted_buffer: Vec::new(),
        }
    }
}

impl<R: Read> Read for RarAesReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.active {
            return self.reader.read(buf);
        }

        // If we have data in buffer, use it first
        if self.buffer_pos < self.buffer.len() {
            let available = self.buffer.len() - self.buffer_pos;
            let to_copy = buf.len().min(available);
            buf[..to_copy]
                .copy_from_slice(&self.buffer[self.buffer_pos..self.buffer_pos + to_copy]);
            self.buffer_pos += to_copy;
            return Ok(to_copy);
        }

        // Reset buffer for new data
        self.buffer.clear();
        self.buffer_pos = 0;

        // Read more encrypted data
        let mut temp_buf = vec![0u8; 4096];
        let bytes_read = self.reader.read(&mut temp_buf)?;

        if bytes_read == 0 {
            return Ok(0);
        }

        // Add new data to encrypted buffer
        self.encrypted_buffer
            .extend_from_slice(&temp_buf[..bytes_read]);

        // Decrypt complete 16-byte blocks
        if let Some(ref mut dec) = self.decryptor {
            let complete_blocks = self.encrypted_buffer.len() / 16;
            if complete_blocks > 0 {
                let blocks_to_decrypt = complete_blocks * 16;

                // Create GenericArray blocks for batch decryption
                let mut blocks: Vec<GenericArray<u8, _>> = self.encrypted_buffer
                    [..blocks_to_decrypt]
                    .chunks_exact(16)
                    .map(|chunk| *GenericArray::from_slice(chunk))
                    .collect();

                // Decrypt all blocks at once to maintain CBC chain
                dec.decrypt_blocks_mut(&mut blocks);

                // Move decrypted data to buffer
                for block in blocks {
                    self.buffer.extend_from_slice(&block);
                }

                // Keep remaining incomplete block for next read
                self.encrypted_buffer.drain(..blocks_to_decrypt);
            }
        }

        // Return data from buffer
        if !self.buffer.is_empty() {
            let to_copy = buf.len().min(self.buffer.len());
            buf[..to_copy].copy_from_slice(&self.buffer[..to_copy]);
            self.buffer_pos = to_copy;
            Ok(to_copy)
        } else {
            // No complete blocks to decrypt yet, try reading more
            self.read(buf)
        }
    }
}

impl<R: Read + Seek> Seek for RarAesReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.buffer.clear();
        self.buffer_pos = 0;
        self.encrypted_buffer.clear();
        self.reader.seek(pos)
    }
}

/// Generate the decryption key from the encryption block infos
fn generate_key(feb: &FileEncryptionBlock, pwd: &str) -> [u8; 32] {
    let iter_number = 2u32.pow(feb.kdf_count.into());
    let mut key = [0u8; 32];
    let _ = pbkdf2::<Hmac<Sha256>>(pwd.as_bytes(), &feb.salt, iter_number, &mut key);
    key
}

#[test]
fn test_aes_stream_disabled() {
    use std::io::Cursor;

    let data = b"Hello World!";
    let cursor = Cursor::new(data);
    let file = FileBlock::default();

    let mut reader = RarAesReader::new(cursor, file, "");
    let mut buf = [0u8; 12];
    let read_bytes = reader.read(&mut buf).unwrap();

    assert_eq!(read_bytes, 12);
    assert_eq!(&buf, data);
}

#[test]
fn test_password_hash_recovery() {
    let pwd = "test";
    // values from encrypted_test.rar
    let salt = [ 207, 73, 219, 137, 241, 60, 48, 114, 55, 34, 150, 43, 219, 93, 243, 85 ];
    let kdf_count = 15u8;
    let pw_check = [ 56, 179, 155, 142, 60, 29, 213, 21 ];

    let iter_number = 2u32.pow(kdf_count.into());
    let mut key_extended = [0u8; 64];
    let _ = pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(pwd.as_bytes(), &salt, iter_number, &mut key_extended);
    
    println!("pw_check target: {:?}", pw_check);
    println!("derived 32..40: {:?}", &key_extended[32..40]);
    println!("derived 0..8: {:?}", &key_extended[0..8]);
}
