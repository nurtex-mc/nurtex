use crate::{CONTINUE_BIT, SEGMENT_BITS, read_byte};

/// Трейт для типа `i32` с варьируемой длинной (в протоколе как `VarInt`)
pub trait VarI32
where
  Self: Sized,
{
  /// Метод чтения `VarI32` из буффера
  fn read_var(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self>;

  /// Метод записи `VarI32` в буффер
  fn write_var(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()>;
}

impl VarI32 for i32 {
  fn read_var(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let mut value = 0i32;
    let mut position = 0u32;

    loop {
      let byte = read_byte(buffer)?;
      value |= (((byte & SEGMENT_BITS) as u32) << position) as i32;

      if (byte & CONTINUE_BIT) == 0 {
        break;
      }

      position += 7;

      if position >= 32 {
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
      array[0] = (value & SEGMENT_BITS as i32) as u8;
      value = (value >> 7) & (i32::MAX >> 6);

      if value != 0 {
        array[0] |= CONTINUE_BIT;
      }

      buffer.write_all(&array)?;
    }

    Ok(())
  }
}
