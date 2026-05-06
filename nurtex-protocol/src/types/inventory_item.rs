use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Предмет инвентаря
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
  Null,
  Some { count: i32, id: i32 },
}

impl Buffer for Item {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    if !bool::read_buf(buffer)? {
      return Some(Self::Null);
    }

    Some(Self::Some {
      count: i32::read_var(buffer)?,
      id: i32::read_var(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    match self {
      Self::Null => {
        false.write_buf(buffer)?;
      }
      Self::Some { count, id } => {
        true.write_buf(buffer)?;
        count.write_var(buffer)?;
        id.write_var(buffer)?;
      }
    }

    Ok(())
  }
}
