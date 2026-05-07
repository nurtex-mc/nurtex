use hashbrown::HashMap;
use tokio::sync::RwLock;

use crate::protocol::types::{BlockPos, Chunk, ChunkPos};
use crate::world::Entity;

/// Хранилище данных
#[derive(Debug)]
pub struct Storage {
  /// Список всех сущностей
  pub entities: RwLock<HashMap<i32, Entity>>,

  /// Список всех чанков
  pub chunks: RwLock<HashMap<ChunkPos, Chunk>>,
}

impl Storage {
  /// Метод создания пустого хранилища
  pub fn null() -> Self {
    Self {
      entities: RwLock::new(HashMap::new()),
      chunks: RwLock::new(HashMap::new()),
    }
  }

  /// Метод добавления сущности в хранилище
  pub async fn add_entity(&self, id: i32, entity: Entity) {
    let mut guard = self.entities.write().await;
    guard.insert(id, entity);
  }

  /// Метод удаления сущности из хранилища
  pub async fn remove_entity(&self, id: &i32) {
    let mut guard = self.entities.write().await;
    guard.remove(id);
  }

  /// Метод получения клона сущности по ID
  pub async fn get_entity(&self, id: &i32) -> Option<Entity> {
    let guard = self.entities.read().await;
    guard.get(id).cloned()
  }

  /// Метод получения количества сущностей
  pub async fn entity_count(&self) -> usize {
    self.entities.read().await.len()
  }

  /// Функция временного захвата сущности
  pub async fn capture_entity<F>(&self, id: &i32, f: F)
  where
    F: AsyncFnOnce(&mut Entity),
  {
    let mut guard = self.entities.write().await;

    if let Some(entity) = guard.get_mut(id) {
      f(entity).await;
    }
  }

  /// Функция временного захвата списка сущностей
  pub async fn capture_entities<F>(&self, f: F)
  where
    F: AsyncFnOnce(&mut HashMap<i32, Entity>),
  {
    let mut guard = self.entities.write().await;
    f(&mut *guard).await;
  }

  /// Метод очитски хранилища
  pub async fn clear(&self) {
    self.entities.write().await.clear();
    self.chunks.write().await.clear();
  }

  /// Метод добавления чанка в хранилище
  pub async fn add_chunk(&self, chunk: Chunk) {
    self.chunks.write().await.insert(chunk.position, chunk);
  }

  /// Метод получения блока по координатам
  pub async fn get_block(&self, pos: BlockPos) -> Option<u32> {
    let chunk_pos = ChunkPos::from_block(pos);
    let guard = self.chunks.read().await;
    let chunk = guard.get(&chunk_pos)?;
    chunk.get_block(pos)
  }

  /// Метод получения количества чанков
  pub async fn chunk_count(&self) -> usize {
    self.chunks.read().await.len()
  }
}
