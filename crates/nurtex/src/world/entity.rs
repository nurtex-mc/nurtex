use std::sync::atomic::{AtomicI32, Ordering};

use crate::protocol::types::{Rotation, Vector3};
use uuid::Uuid;

/// Идентификатор сущности в мире
pub struct EntityId(AtomicI32);

impl EntityId {
  /// Метод создания отрицательного ID (-1)
  pub fn negative() -> Self {
    Self(AtomicI32::new(-1))
  }

  /// Метод получения ID
  pub fn get(&self) -> i32 {
    self.0.load(Ordering::SeqCst)
  }

  /// Метод установки ID
  pub fn set(&self, entity_id: i32) {
    self.0.store(entity_id, Ordering::SeqCst);
  }
}

/// Сущность мира
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
  pub entity_type: i32,
  pub entity_uuid: Uuid,
  pub position: Vector3,
  pub rotation: Rotation,
  pub velocity: Vector3,
  pub on_ground: bool,
}

impl Default for Entity {
  fn default() -> Self {
    Self {
      entity_type: -1,
      entity_uuid: Uuid::nil(),
      position: Vector3::zero(),
      rotation: Rotation::zero(),
      velocity: Vector3::zero(),
      on_ground: false,
    }
  }
}
