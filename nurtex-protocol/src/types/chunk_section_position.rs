use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Позиция секции чанка (16x16x16 блоков)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkSectionPosition {
  pub x: i32,
  pub y: i32,
  pub z: i32,
}

impl ChunkSectionPosition {
  /// Метод создания нового экземпляра `ChunkSectionPosition`
  pub fn new(x: i32, y: i32, z: i32) -> Self {
    Self { x, y, z }
  }

  /// Метод кодировки позиции секции чанка
  pub fn encode(&self) -> i64 {
    ((self.x as i64 & 0x3FFFFF) << 42) | ((self.z as i64 & 0x3FFFFF) << 20) | (self.y as i64 & 0xFFFFF)
  }

  /// Метод декодировки позиции секции чанка
  pub fn decode(value: i64) -> Self {
    let x = (value >> 42) as i32;
    let z = ((value >> 20) & 0x3FFFFF) as i32;
    let y = (value & 0xFFFFF) as i32;

    let x = if x & 0x200000 != 0 { x | !0x3FFFFF } else { x };
    let z = if z & 0x200000 != 0 { z | !0x3FFFFF } else { z };
    let y = if y & 0x80000 != 0 { y | !0xFFFFF } else { y };

    Self { x, z, y }
  }
}

impl Buffer for ChunkSectionPosition {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let value = i64::read_buf(buffer)?;
    Some(Self::decode(value))
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.encode().write_buf(buffer)
  }
}
