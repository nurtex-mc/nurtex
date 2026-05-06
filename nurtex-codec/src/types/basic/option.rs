use crate::Buffer;

impl<T: Buffer> Buffer for Option<T> {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    if bool::read_buf(buffer)? { Some(T::read_buf(buffer)) } else { None }
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    if let Some(s) = self {
      true.write_buf(buffer)?;
      s.write_buf(buffer)
    } else {
      false.write_buf(buffer)
    }
  }
}
