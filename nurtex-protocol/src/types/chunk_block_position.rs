use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Позиция блока внутри чанка
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkBlockPosition {
  pub x: u8,
  pub y: u8,
  pub z: u8,
}

impl ChunkBlockPosition {
  /// Метод создания нового экземпляра `ChunkBlockPosition`
  pub fn new(x: u8, y: u8, z: u8) -> Self {
    Self { x, y, z }
  }

  /// Метод кодировки позиции блока относительно чанку
  pub fn encode(&self) -> u16 {
    ((self.x as u16 & 0xF) << 8) | ((self.z as u16 & 0xF) << 4) | (self.y as u16 & 0xF)
  }

  /// Метод декодировки позиции блока относительно чанку
  pub fn decode(value: u16) -> Self {
    let x = ((value >> 8) & 0xF) as u8;
    let z = ((value >> 4) & 0xF) as u8;
    let y = (value & 0xF) as u8;

    Self { x, y, z }
  }
}

impl Buffer for ChunkBlockPosition {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let value = u16::read_buf(buffer)?;
    Some(Self::decode(value))
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.encode().write_buf(buffer)
  }
}
