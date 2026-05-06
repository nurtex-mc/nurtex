use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Структура дополнительной информации о сообщении
#[derive(Debug, Clone, PartialEq)]
pub struct AdditionalMessageInfo {
  pub message_count: i32,
  pub acknowledged: [u8; 3],
  pub checksum: u8,
}

impl Default for AdditionalMessageInfo {
  fn default() -> Self {
    Self {
      message_count: 0,
      acknowledged: [0; 3],
      checksum: 0,
    }
  }
}

impl Buffer for AdditionalMessageInfo {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      message_count: i32::read_var(buffer)?,
      acknowledged: {
        let mut array = [0u8; 3];
        for byte in &mut array {
          *byte = u8::read_buf(buffer)?;
        }
        array
      },
      checksum: u8::read_buf(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.message_count.write_var(buffer)?;

    for byte in &self.acknowledged {
      byte.write_buf(buffer)?;
    }

    self.checksum.write_buf(buffer)?;

    Ok(())
  }
}
