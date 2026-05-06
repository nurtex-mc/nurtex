use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Состояние ресурс пака
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResourcePackState {
  SuccessfullyLoaded,
  Declined,
  FailedDownload,
  Accepted,
  Downloaded,
  InvalidUrl,
  FailedToReload,
  Discarded,
}

impl Buffer for ResourcePackState {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      0 => Self::SuccessfullyLoaded,
      1 => Self::Declined,
      2 => Self::FailedDownload,
      3 => Self::Accepted,
      4 => Self::Downloaded,
      5 => Self::InvalidUrl,
      6 => Self::FailedToReload,
      7 => Self::Discarded,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::SuccessfullyLoaded => 0,
      Self::Declined => 1,
      Self::FailedDownload => 2,
      Self::Accepted => 3,
      Self::Downloaded => 4,
      Self::InvalidUrl => 5,
      Self::FailedToReload => 6,
      Self::Discarded => 7,
    };

    id.write_var(buffer)
  }
}
