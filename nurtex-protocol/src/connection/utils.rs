use crate::packets::login::{ClientsideEncryptionRequest, ServersideEncryptionResponse};

/// Функция обработки запроса шифрования
pub fn handle_encryption_request(request: &ClientsideEncryptionRequest) -> Option<(ServersideEncryptionResponse, [u8; 16])> {
  if let Some(encryption) = nurtex_encrypt::try_encrypt(&request.public_key, &request.verify_token) {
    let response = ServersideEncryptionResponse {
      shared_secret: encryption.encrypted_public_key,
      verify_token: encryption.encrypted_challenge,
    };

    Some((response, encryption.secret_key))
  } else {
    None
  }
}
