use crate::Buffer;

impl Buffer for bool {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let byte = u8::read_buf(buffer)?;

    if byte > 1 {
      return None;
    }

    Some(byte != 0)
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let byte = u8::from(*self);
    byte.write_buf(buffer)
  }
}
