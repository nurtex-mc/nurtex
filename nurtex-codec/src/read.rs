use std::io::Read;

/// Вспомогательная функция для чтения одного байта из буффера
pub fn read_byte(buffer: &mut std::io::Cursor<&[u8]>) -> Option<u8> {
  let mut buf = [0u8; 1];
  buffer.read_exact(&mut buf).ok()?;
  Some(buf[0])
}

/// Вспомогательная функция чтения байтов из буффера
pub fn read_bytes<'a>(buffer: &'a mut std::io::Cursor<&[u8]>, length: usize) -> Option<&'a [u8]> {
  if length > (buffer.get_ref().len() - buffer.position() as usize) {
    return None;
  }

  let initial_position = buffer.position() as usize;
  buffer.set_position(buffer.position() + length as u64);
  let data = &buffer.get_ref()[initial_position..initial_position + length];

  Some(data)
}
