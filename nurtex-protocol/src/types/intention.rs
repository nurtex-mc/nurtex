use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClientIntention {
  Status,
  Login,
}

impl Buffer for ClientIntention {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      1 => Self::Status,
      2 => Self::Login,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::Status => 1,
      Self::Login => 2,
    };

    id.write_var(buffer)
  }
}
