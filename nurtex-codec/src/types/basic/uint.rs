use crate::Buffer;

use byteorder::{BE, ReadBytesExt, WriteBytesExt};

impl Buffer for u8 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_u8().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_u8(*self)
  }
}

impl Buffer for u16 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_u16::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_u16::<BE>(*self)
  }
}

impl Buffer for u32 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_u32::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_u32::<BE>(*self)
  }
}

impl Buffer for u64 {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    buffer.read_u64::<BE>().ok()
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_u64::<BE>(*self)
  }
}
