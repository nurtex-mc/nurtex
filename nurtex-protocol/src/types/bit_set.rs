use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Набор битов
#[derive(Clone, Debug, PartialEq)]
pub struct BitSet(Vec<i64>);

impl BitSet {
  /// Метод получения набора
  pub fn get(&self) -> &Vec<i64> {
    &self.0
  }
}

impl Buffer for BitSet {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let len = i32::read_var(buffer)? as usize;
    let mut set = Vec::with_capacity(len);

    for _ in 0..len {
      set.push(i64::read_buf(buffer)?);
    }

    Some(Self(set))
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    (self.0.len() as i32).write_var(buffer)?;

    for el in &self.0 {
      el.write_buf(buffer)?;
    }

    Ok(())
  }
}
