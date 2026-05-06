use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Структура позиции блока
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockPosition {
  pub x: i32,
  pub y: i16,
  pub z: i32,
}

impl BlockPosition {
  /// Метод создания нового экземпляра `BlockPosition`
  pub fn new(x: i32, y: i16, z: i32) -> Self {
    Self { x, y, z }
  }

  /// Метод создания нулевой позиции блока
  pub fn zero() -> Self {
    Self { x: 0, y: 0, z: 0 }
  }
}

impl Buffer for BlockPosition {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      x: i32::read_buf(buffer)?,
      y: i16::read_buf(buffer)?,
      z: i32::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    self.x.write_buf(buffer)?;
    self.y.write_buf(buffer)?;
    self.z.write_buf(buffer)?;
    Ok(())
  }
}
