use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Тип взаимодействия
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum InteractType {
  Interact,
  Attack,
  InteractAt,
}

impl Buffer for InteractType {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      0 => Self::Interact,
      1 => Self::Attack,
      2 => Self::InteractAt,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::Interact => 0,
      Self::Attack => 1,
      Self::InteractAt => 2,
    };

    id.write_var(buffer)?;

    Ok(())
  }
}
