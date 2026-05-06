use crate::Buffer;

use byteorder::{BE, ReadBytesExt, WriteBytesExt};

impl Buffer for f32 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_f32::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_f32::<BE>(*self)
  }
}

impl Buffer for f64 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_f64::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_f64::<BE>(*self)
  }
}
