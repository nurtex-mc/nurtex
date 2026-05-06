use crate::Buffer;
use crate::types::variable::VarI32;

impl<T: Buffer> Buffer for Vec<T> {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let length = i32::read_var(buffer)? as usize;
    let mut contents = Vec::with_capacity(usize::min(length, 65536));

    for _ in 0..length {
      contents.push(T::read_buf(buffer)?);
    }

    Some(contents)
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    (self.len() as i32).write_var(buffer)?;

    for item in self.iter() {
      T::write_buf(item, buffer)?;
    }

    Ok(())
  }
}
