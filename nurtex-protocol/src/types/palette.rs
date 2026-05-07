use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Формат палитры
#[derive(Clone, Debug, PartialEq)]
pub enum PaletteFormat {
  Single(u32),
  Direct,
  Indirect(Vec<u32>),
}

/// Название палитры
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKind {
  Blocks,
  Biomes,
}

/// Палитра
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
  pub bits_per_entry: u8,
  pub format: PaletteFormat,
  pub data: Vec<u64>,
  pub value_count: usize,
}

impl Palette {
  /// Метод получения элемента по индексу
  pub fn get_by_index(&self, index: usize) -> Option<u32> {
    if index >= self.value_count {
      return None;
    }

    match &self.format {
      PaletteFormat::Single(id) => Some(*id),
      PaletteFormat::Direct => Some(self.storage_get(index)? as u32),
      PaletteFormat::Indirect(palette) => {
        let pal_index = self.storage_get(index)? as usize;
        palette.get(pal_index).copied()
      }
    }
  }

  fn storage_get(&self, index: usize) -> Option<u64> {
    let bits = self.bits_per_entry as usize;

    if bits == 0 {
      return Some(0);
    }

    if !(1..=32).contains(&bits) {
      return None;
    }

    let values_per_word = 64 / bits;

    if values_per_word == 0 {
      return None;
    }

    let word_index = index / values_per_word;
    let idx_in_word = index - word_index * values_per_word;
    let shift = idx_in_word * bits;
    let mask = (1u64 << bits) - 1;
    let word = *self.data.get(word_index)?;

    Some((word >> shift) & mask)
  }
}

/// Функция чтения палитры из буффера
pub fn read_palette(buffer: &mut std::io::Cursor<&[u8]>, value_count: usize, kind: PaletteKind) -> Option<Palette> {
  let bits_per_entry = u8::read_buf(buffer)?;

  let format = match (kind, bits_per_entry) {
    (_, 0) => PaletteFormat::Single(i32::read_var(buffer)? as u32),
    (PaletteKind::Blocks, 1..=8) => {
      let len = i32::read_var(buffer)? as usize;
      let mut entries = Vec::with_capacity(len);
      for _ in 0..len {
        entries.push(i32::read_var(buffer)? as u32);
      }
      PaletteFormat::Indirect(entries)
    }
    (PaletteKind::Biomes, 1..=3) => {
      let len = i32::read_var(buffer)? as usize;
      let mut entries = Vec::with_capacity(len);
      for _ in 0..len {
        entries.push(i32::read_var(buffer)? as u32);
      }
      PaletteFormat::Indirect(entries)
    }
    _ => PaletteFormat::Direct,
  };

  let bits = bits_per_entry as usize;
  let values_per_word = if bits == 0 { 0 } else { 64 / bits };
  let word_count = if bits == 0 || values_per_word == 0 { 0 } else { value_count.div_ceil(values_per_word) };

  let mut data = Vec::with_capacity(word_count);

  for _ in 0..word_count {
    data.push(u64::read_buf(buffer)?);
  }

  Some(Palette {
    bits_per_entry,
    format,
    data,
    value_count,
  })
}
