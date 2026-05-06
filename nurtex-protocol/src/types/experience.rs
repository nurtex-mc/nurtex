use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Структура опыта
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Experience {
  pub bar: f32,
  pub level: i32,
  pub total: i32,
}

impl Default for Experience {
  fn default() -> Self {
    Self { bar: 0.0, level: 0, total: 0 }
  }
}

impl Buffer for Experience {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    Some(Self {
      bar: f32::read_buf(buffer)?,
      level: i32::read_var(buffer)?,
      total: i32::read_var(buffer)?,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.bar.write_buf(buffer)?;
    self.level.write_var(buffer)?;
    self.total.write_var(buffer)?;
    Ok(())
  }
}
