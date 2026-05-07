use crate::read_byte;

/// Трейт для типа `i64` с варьируемой длинной (в протоколе как `VarLong`)
pub trait VarI64
where
  Self: Sized,
{
  /// Метод чтения `VarI64` из буффера
  fn read_var(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self>;

  /// Метод записи `VarI64` в буффер
  fn write_var(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl VarI64 for i64 {
  fn read_var(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let mut value = 0i64;
    let mut position = 0u32;

    loop {
      let byte = read_byte(buffer)?;
      value |= (((byte & 0x7F) as u32) << position) as i64;

      if (byte & 0x80) == 0 {
        break;
      }

      position += 7;

      if position >= 64 {
        return None;
      }
    }

    Some(value)
  }

  fn write_var(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let mut array = [0];
    let mut value = *self;

    if value == 0 {
      buffer.write_all(&array)?;
      return Ok(());
    }

    while value != 0 {
      array[0] = (value & 0x7F) as u8;
      value = (value >> 7) & (i64::MAX >> 6);

      if value != 0 {
        array[0] |= 0x80;
      }

      buffer.write_all(&array)?;
    }

    Ok(())
  }
}
