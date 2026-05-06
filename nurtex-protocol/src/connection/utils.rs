use nurtex_encrypt::{digest_data, encrypt};

use crate::packets::login::{ClientsideEncryptionRequest, ServersideEncryptionResponse};

/// Функция обработки запроса шифрования
pub fn handle_encryption_request(request: &ClientsideEncryptionRequest) -> Option<(ServersideEncryptionResponse, [u8; 16])> {
  if let Some(encryption) = encrypt(&request.public_key, &request.verify_token) {
    let response = ServersideEncryptionResponse {
      shared_secret: encryption.encrypted_public_key,
      verify_token: encryption.encrypted_challenge,
    };

    Some((response, encryption.secret_key))
  } else {
    None
  }
}

/// Функция получения хэша сервера
pub fn get_server_hash(server_id: &str, shared_secret: &[u8; 16], public_key: &[u8]) -> String {
  let hash = digest_data(server_id.as_bytes(), public_key, shared_secret);
  create_minecraft_hash(&hash)
}

/// Функция создания Minecraft хэша
fn create_minecraft_hash(hash: &[u8]) -> String {
  let is_negative = (hash[0] & 0x80) == 0x80;

  let hash_value = if is_negative {
    let mut result = Vec::with_capacity(hash.len());
    let mut carry = true;

    for &byte in hash.iter().rev() {
      let inverted = !byte;

      if carry {
        let (sum, overflow) = inverted.overflowing_add(1);
        result.push(sum);
        carry = overflow;
      } else {
        result.push(inverted);
      }
    }

    result.reverse();

    result
  } else {
    hash.to_vec()
  };

  let hex = hash_value.iter().map(|b| format!("{:02x}", b)).collect::<String>();

  let trimmed = hex.trim_start_matches('0');

  if is_negative {
    format!("-{}", if trimmed.is_empty() { "0" } else { trimmed })
  } else {
    if trimmed.is_empty() { "0" } else { trimmed }.to_string()
  }
}
