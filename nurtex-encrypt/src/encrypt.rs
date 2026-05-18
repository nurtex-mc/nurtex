use aes::Aes128;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, inout::InOutBuf};
use rand::{Rng, thread_rng};

pub type AesEncryptor = cfb8::Encryptor<Aes128>;
pub type AesDecryptor = cfb8::Decryptor<Aes128>;

#[derive(Debug)]
pub struct EncryptResult {
  pub secret_key: [u8; 16],
  pub encrypted_public_key: Vec<u8>,
  pub encrypted_challenge: Vec<u8>,
}

/// Функция генерации секретного ключа
fn generate_secret_key() -> [u8; 16] {
  let mut key = [0u8; 16];
  thread_rng().fill(&mut key);
  key
}

/// Метод попытки создания шифрования на основе публичного ключа и челленжда
pub fn try_encrypt(public_key: &[u8], challenge: &[u8]) -> Option<EncryptResult> {
  let secret_key = generate_secret_key();

  let encrypted_public_key = rsa_public_encrypt_pkcs1::encrypt(public_key, &secret_key).ok()?;
  let encrypted_challenge = rsa_public_encrypt_pkcs1::encrypt(public_key, challenge).ok()?;

  Some(EncryptResult {
    secret_key,
    encrypted_public_key,
    encrypted_challenge,
  })
}

/// Функция создания AES-шифра
pub fn create_cipher(key: &[u8]) -> (AesEncryptor, AesDecryptor) {
  (AesEncryptor::new_from_slices(key, key).unwrap(), AesDecryptor::new_from_slices(key, key).unwrap())
}

/// Функция шифрования данных
pub fn encrypt_data(cipher: &mut AesEncryptor, buf: &mut [u8]) {
  let (blocks, _) = InOutBuf::from(buf).into_chunks();
  cipher.encrypt_blocks_inout_mut(blocks);
}

/// Функция расшифровки данных
pub fn decrypt_data(cipher: &mut AesDecryptor, buf: &mut [u8]) {
  let (blocks, _) = InOutBuf::from(buf).into_chunks();
  cipher.decrypt_blocks_inout_mut(blocks);
}
