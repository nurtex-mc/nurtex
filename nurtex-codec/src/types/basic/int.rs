use crate::Buffer;

use byteorder::{BE, ReadBytesExt, WriteBytesExt};

impl Buffer for i8 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i8().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_i8(*self)
  }
}

impl Buffer for i16 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i16::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_i16::<BE>(*self)
  }
}

impl Buffer for i32 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i32::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_i32::<BE>(*self)
  }
}

impl Buffer for i64 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_i64::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_i64::<BE>(*self)
  }
}
