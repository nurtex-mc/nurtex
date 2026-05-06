use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Статус видимости партиклов
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ParticleStatus {
  All,
  Decreased,
  Minimal,
}

impl Buffer for ParticleStatus {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      0 => Self::All,
      1 => Self::Decreased,
      2 => Self::Minimal,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::All => 0,
      Self::Decreased => 1,
      Self::Minimal => 2,
    };

    id.write_var(buffer)
  }
}
