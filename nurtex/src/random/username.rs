use rand::{Rng, thread_rng};

/// Набор допустимых символов для юзернейма
const USERNAME_SAMPLE: &[u8] = b"qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM1234567890_";

/// Функция генерации юзернейма с указанной длиной (максимальная 16, минимальная 3)
pub fn generate_username(length: usize) -> String {
  let pretty_length = {
    if length < 3 {
      3
    } else if length > 16 {
      16
    } else {
      length
    }
  };

  let username: String = (0..pretty_length)
    .map(|_| {
      let idx = thread_rng().gen_range(0..USERNAME_SAMPLE.len());
      char::from(USERNAME_SAMPLE[idx])
    })
    .collect();

  username
}
