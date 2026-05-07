use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;
use nurtex_codec::{read_byte, read_bytes};

/// Сырые данные карт высот
#[derive(Clone, Debug, PartialEq)]
pub struct HeightmapsRaw {
  pub bytes: Vec<u8>,
}

/// Сырые данные NBT
#[derive(Clone, Debug, PartialEq)]
pub struct NbtRaw {
  pub bytes: Vec<u8>,
}

impl HeightmapsRaw {
  fn remaining(buffer: &std::io::Cursor<&[u8]>) -> usize {
    buffer.get_ref().len().saturating_sub(buffer.position() as usize)
  }

  fn read_nbt_raw(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Vec<u8>> {
    let start = buffer.position() as usize;

    let tag_id = read_byte(buffer)?;
    if tag_id != 0x0A {
      return None;
    }

    let name_len = u16::read_buf(buffer)? as usize;
    let _ = read_bytes(buffer, name_len)?;

    loop {
      let id = read_byte(buffer)?;
      if id == 0 {
        break;
      }

      let name_len = u16::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, name_len)?;
      skip_nbt_payload(buffer, id)?;
    }

    let end = buffer.position() as usize;
    Some(buffer.get_ref()[start..end].to_vec())
  }

  fn read_map_raw(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Vec<u8>> {
    let start = buffer.position() as usize;

    let map_size = i32::read_var(buffer)? as usize;
    for _ in 0..map_size {
      let _key = i32::read_var(buffer)?;
      let len = i32::read_var(buffer)? as usize;
      for _ in 0..len {
        let _ = i64::read_buf(buffer)?;
      }
    }

    let end = buffer.position() as usize;
    Some(buffer.get_ref()[start..end].to_vec())
  }
}

impl Buffer for HeightmapsRaw {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    if Self::remaining(buffer) == 0 {
      return Some(Self { bytes: vec![] });
    }

    let pos = buffer.position() as usize;
    let first = *buffer.get_ref().get(pos)?;

    let bytes = if first == 0x0A { Self::read_nbt_raw(buffer)? } else { Self::read_map_raw(buffer)? };

    Some(Self { bytes })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_all(&self.bytes)
  }
}

impl Buffer for NbtRaw {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    if HeightmapsRaw::remaining(buffer) == 0 {
      return Some(Self { bytes: vec![] });
    }

    let start = buffer.position() as usize;
    let tag_id = read_byte(buffer)?;
    if tag_id == 0 {
      let end = buffer.position() as usize;
      return Some(Self {
        bytes: buffer.get_ref()[start..end].to_vec(),
      });
    }

    let name_len = u16::read_buf(buffer)? as usize;
    let _ = read_bytes(buffer, name_len)?;
    skip_nbt_payload(buffer, tag_id)?;

    let end = buffer.position() as usize;
    Some(Self {
      bytes: buffer.get_ref()[start..end].to_vec(),
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    buffer.write_all(&self.bytes)
  }
}

/// Функция пропуска NBT данных
fn skip_nbt_payload(buffer: &mut std::io::Cursor<&[u8]>, tag_id: u8) -> Option<()> {
  match tag_id {
    1 => {
      let _ = i8::read_buf(buffer)?;
    }
    2 => {
      let _ = i16::read_buf(buffer)?;
    }
    3 => {
      let _ = i32::read_buf(buffer)?;
    }
    4 => {
      let _ = i64::read_buf(buffer)?;
    }
    5 => {
      let _ = f32::read_buf(buffer)?;
    }
    6 => {
      let _ = f64::read_buf(buffer)?;
    }
    7 => {
      let len = i32::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, len)?;
    }
    8 => {
      let len = u16::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, len)?;
    }
    9 => {
      let child_id = read_byte(buffer)?;
      let len = i32::read_buf(buffer)? as usize;
      for _ in 0..len {
        skip_nbt_payload(buffer, child_id)?;
      }
    }
    10 => loop {
      let id = read_byte(buffer)?;
      if id == 0 {
        break;
      }
      let name_len = u16::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, name_len)?;
      skip_nbt_payload(buffer, id)?;
    },
    11 => {
      let len = i32::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, len * 4)?;
    }
    12 => {
      let len = i32::read_buf(buffer)? as usize;
      let _ = read_bytes(buffer, len * 8)?;
    }
    _ => return None,
  }

  Some(())
}
