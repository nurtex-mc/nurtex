use std::hash::{Hash, Hasher};

use nurtex_codec::Buffer;

use crate::types::BlockPos;

use super::{ChunkData, Palette, PaletteKind, read_palette};

/// Мировой чанк
#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
  /// Позиция чанка
  pub position: ChunkPos,

  /// Минимальная высота
  pub min_y: i32,

  /// Список секций чанка
  pub sections: Vec<ChunkSection>,
}

/// Позиция чанка
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
  pub x: i32,
  pub z: i32,
}

impl ChunkPos {
  /// Метод создания новой позиции чанка
  pub fn new(x: i32, z: i32) -> Self {
    Self { x, z }
  }
}

impl Hash for ChunkPos {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.x.hash(state);
    self.z.hash(state);
  }
}

impl ChunkPos {
  pub fn from_block(pos: BlockPos) -> Self {
    Self {
      x: pos.x.div_euclid(16),
      z: pos.z.div_euclid(16),
    }
  }
}

/// Секция чанка
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkSection {
  /// Количество блоков, не относящихся к воздуху
  pub non_air_block_count: u16,

  /// Блоки чанка (всего 4096)
  pub blocks: Palette,

  /// Биомы чанка
  pub biomes: Palette,
}

impl Chunk {
  /// Декодировка секций чанка
  pub fn decode(chunk_x: i32, chunk_z: i32, chunk_data: &ChunkData, section_count: usize, min_y: i32) -> Option<Self> {
    let mut cur = std::io::Cursor::new(&chunk_data.sections[..]);
    let mut sections = Vec::with_capacity(section_count);

    for _ in 0..section_count {
      let non_air_block_count = u16::read_buf(&mut cur)?;
      let blocks = read_palette(&mut cur, 16 * 16 * 16, PaletteKind::Blocks)?;
      let biomes = read_palette(&mut cur, 4 * 4 * 4, PaletteKind::Biomes)?;

      sections.push(ChunkSection {
        non_air_block_count,
        blocks,
        biomes,
      });
    }

    Some(Self {
      position: ChunkPos::new(chunk_x, chunk_z),
      min_y: min_y,
      sections: sections,
    })
  }

  /// Декодировка секций чанка до конца данныхs
  pub fn decode_to_end(chunk_x: i32, chunk_z: i32, chunk_data: &ChunkData, min_y: i32) -> Option<Self> {
    let mut cur = std::io::Cursor::new(&chunk_data.sections[..]);
    let mut sections = Vec::new();

    while (cur.position() as usize) < cur.get_ref().len() {
      let before = cur.position();

      let non_air_block_count = u16::read_buf(&mut cur)?;
      let blocks = read_palette(&mut cur, 16 * 16 * 16, PaletteKind::Blocks)?;
      let biomes = read_palette(&mut cur, 4 * 4 * 4, PaletteKind::Biomes)?;

      sections.push(ChunkSection {
        non_air_block_count,
        blocks,
        biomes,
      });

      if cur.position() == before {
        return None;
      }
    }

    Some(Self {
      position: ChunkPos::new(chunk_x, chunk_z),
      min_y: min_y,
      sections: sections,
    })
  }

  /// Метод получения блока по мировым координатам
  pub fn get_block(&self, pos: BlockPos) -> Option<u32> {
    if pos.y < self.min_y {
      return None;
    }

    let rel_y = (pos.y - self.min_y) as usize;
    let section_index = rel_y / 16;
    let local_y = rel_y % 16;
    let local_x = pos.x.rem_euclid(16) as usize;
    let local_z = pos.z.rem_euclid(16) as usize;

    let section = self.sections.get(section_index)?;

    section.blocks.get_by_index((local_y * 16 * 16) + (local_z * 16) + local_x)
  }

  /// Метод получения блока по координатам относительно текущего чанка
  pub fn get_block_relative(&self, x: usize, y: usize, z: usize) -> Option<u32> {
    if x >= 16 || z >= 16 {
      return None;
    }

    let section_index = y / 16;
    let local_y = y % 16;
    let section = self.sections.get(section_index)?;
    let index = (local_y * 16 * 16) + (z * 16) + x;

    section.blocks.get_by_index(index)
  }
}
