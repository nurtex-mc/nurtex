use std::io::{self, Cursor, Write};

use nurtex_codec::Buffer;

/// Направление лица
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Face {
  Bottom,
  Top,
  North,
  South,
  West,
  East,
}

impl Buffer for Face {
  fn read_buf(buffer: &mut Cursor<&[u8]>) -> Option<Self> {
    let id = i8::read_buf(buffer)?;

    Some(match id {
      0 => Self::Bottom,
      1 => Self::Top,
      2 => Self::North,
      3 => Self::South,
      4 => Self::West,
      5 => Self::East,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl Write) -> io::Result<()> {
    let id = match self {
      Self::Bottom => 0,
      Self::Top => 1,
      Self::North => 2,
      Self::South => 3,
      Self::West => 4,
      Self::East => 5,
    };

    (id as i8).write_buf(buffer)?;

    Ok(())
  }
}
