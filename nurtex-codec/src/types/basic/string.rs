use crate::types::variable::VarI32;
use crate::{Buffer, read_bytes};

impl Buffer for String {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let length = i32::read_var(buffer)? as u32;

    if length > 32767 * 4 {
      return None;
    }

    let buffer = read_bytes(buffer, length as usize)?;
    let string = std::str::from_utf8(buffer).ok()?;

    if string.len() > length as usize {
      return None;
    }

    Some(string.to_string())
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let len = self.len();

    if len > 32767 {
      return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "string is too long"));
    }

    (len as i32).write_var(buffer)?;
    buffer.write_all(self.as_bytes())?;

    Ok(())
  }
}
