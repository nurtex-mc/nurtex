use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Режим чата
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ChatMode {
  Enabled,
  CommandsOnly,
  Hidden,
}

impl Buffer for ChatMode {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      0 => Self::Enabled,
      1 => Self::CommandsOnly,
      2 => Self::Hidden,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::Enabled => 0,
      Self::CommandsOnly => 1,
      Self::Hidden => 2,
    };

    id.write_var(buffer)
  }
}
