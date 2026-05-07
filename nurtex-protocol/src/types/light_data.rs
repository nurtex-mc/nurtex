use nurtex_codec::Buffer;
use nurtex_codec::read_bytes;
use nurtex_codec::types::variable::VarI32;

use super::BitSet;

/// Массив данных света
#[derive(Clone, Debug, PartialEq)]
pub struct LightArray {
  pub bytes: Vec<u8>,
}

impl Buffer for LightArray {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let len = i32::read_var(buffer)? as usize;
    let data = read_bytes(buffer, len)?.to_vec();
    Some(Self { bytes: data })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    (self.bytes.len() as i32).write_var(buffer)?;
    buffer.write_all(&self.bytes)
  }
}

/// Данные света
#[derive(Clone, Debug, PartialEq)]
pub struct LightData {
  pub sky_y_mask: BitSet,
  pub block_y_mask: BitSet,
  pub empty_sky_y_mask: BitSet,
  pub empty_block_y_mask: BitSet,
  pub sky_updates: Vec<LightArray>,
  pub block_updates: Vec<LightArray>,
}

impl Buffer for LightData {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let sky_y_mask = BitSet::read_buf(buffer)?;
    let block_y_mask = BitSet::read_buf(buffer)?;
    let empty_sky_y_mask = BitSet::read_buf(buffer)?;
    let empty_block_y_mask = BitSet::read_buf(buffer)?;

    let sky_count = i32::read_var(buffer)? as usize;
    let mut sky_updates = Vec::with_capacity(sky_count);

    for _ in 0..sky_count {
      sky_updates.push(LightArray::read_buf(buffer)?);
    }

    let block_count = i32::read_var(buffer)? as usize;
    let mut block_updates = Vec::with_capacity(block_count);

    for _ in 0..block_count {
      block_updates.push(LightArray::read_buf(buffer)?);
    }

    Some(Self {
      sky_y_mask,
      block_y_mask,
      empty_sky_y_mask,
      empty_block_y_mask,
      sky_updates,
      block_updates,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    self.sky_y_mask.write_buf(buffer)?;
    self.block_y_mask.write_buf(buffer)?;
    self.empty_sky_y_mask.write_buf(buffer)?;
    self.empty_block_y_mask.write_buf(buffer)?;

    (self.sky_updates.len() as i32).write_var(buffer)?;

    for arr in &self.sky_updates {
      arr.write_buf(buffer)?;
    }

    (self.block_updates.len() as i32).write_var(buffer)?;

    for arr in &self.block_updates {
      arr.write_buf(buffer)?;
    }

    Ok(())
  }
}
