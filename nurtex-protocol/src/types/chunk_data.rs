use nurtex_codec::Buffer;
use nurtex_codec::read_bytes;
use nurtex_codec::types::variable::VarI32;

use super::HeightmapsRaw;

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkData {
  /// Карты высот
  pub heightmaps: HeightmapsRaw,

  /// Сырые данные секций
  pub sections: Vec<u8>,
}

impl Buffer for ChunkData {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      heightmaps: HeightmapsRaw::read_buf(buffer)?,
      sections: {
        let length = i32::read_var(buffer)? as usize;
        read_bytes(buffer, length)?.to_vec()
      },
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.heightmaps.write_buf(buffer)?;
    (self.sections.len() as i32).write_var(buffer)?;

    Ok(())
  }
}
