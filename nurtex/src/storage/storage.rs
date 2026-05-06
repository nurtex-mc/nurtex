use std::sync::Arc;

use hashbrown::HashMap;
use tokio::sync::RwLock;

use crate::world::Entity;

/// Хранилище данных
#[derive(Debug, Clone)]
pub struct Storage {
  /// Список всех сущностей
  pub entities: Arc<RwLock<HashMap<i32, Entity>>>,
}

impl Storage {
  /// Метод создания пустого хранилища
  pub fn null() -> Self {
    Self {
      entities: Arc::new(RwLock::new(HashMap::new())),
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

  /// Метод получения клона сущности
  pub async fn get_entity(&self, id: &i32) -> Option<Entity> {
    let guard = self.entities.read().await;
    guard.get(id).cloned()
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
  }
}
