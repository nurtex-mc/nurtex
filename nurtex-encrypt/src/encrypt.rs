use aes::Aes128;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, inout::InOutBuf};
use rand::{Rng, thread_rng};
use sha1::{Digest, Sha1};

pub type AesEncryptor = cfb8::Encryptor<Aes128>;
pub type AesDecryptor = cfb8::Decryptor<Aes128>;

fn generate_secret_key() -> [u8; 16] {
  let mut key = [0u8; 16];
  thread_rng().fill(&mut key);
  key
}

pub fn digest_data(server_id: &[u8], public_key: &[u8], private_key: &[u8]) -> Vec<u8> {
  let mut digest = Sha1::new();
  digest.update(server_id);
  digest.update(private_key);
  digest.update(public_key);
  digest.finalize().to_vec()
}

#[derive(Debug)]
pub struct EncryptResult {
  pub secret_key: [u8; 16],
  pub encrypted_public_key: Vec<u8>,
  pub encrypted_challenge: Vec<u8>,
}

pub fn encrypt(public_key: &[u8], challenge: &[u8]) -> Option<EncryptResult> {
  let secret_key = generate_secret_key();

  let encrypted_public_key = rsa_public_encrypt_pkcs1::encrypt(public_key, &secret_key).ok()?;
  let encrypted_challenge = rsa_public_encrypt_pkcs1::encrypt(public_key, challenge).ok()?;

  Some(EncryptResult {
    secret_key,
    encrypted_public_key,
    encrypted_challenge,
  })
}

pub fn create_cipher(key: &[u8]) -> (AesEncryptor, AesDecryptor) {
  (AesEncryptor::new_from_slices(key, key).unwrap(), AesDecryptor::new_from_slices(key, key).unwrap())
}

pub fn encrypt_packet(cipher: &mut AesEncryptor, packet: &mut [u8]) {
  let (chunks, _) = InOutBuf::from(packet).into_chunks();
  cipher.encrypt_blocks_inout_mut(chunks);
}

pub fn decrypt_packet(cipher: &mut AesDecryptor, packet: &mut [u8]) {
  let (chunks, _) = InOutBuf::from(packet).into_chunks();
  cipher.decrypt_blocks_inout_mut(chunks);
}
