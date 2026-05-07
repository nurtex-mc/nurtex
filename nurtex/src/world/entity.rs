use std::sync::atomic::{AtomicI32, Ordering};

use crate::protocol::types::{Rotation, Vector3};
use crate::registry::EntityKind;

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
  /// Название сущности
  pub kind: EntityKind,

  /// UUID сущности
  pub uuid: Uuid,

  /// Позиция сущности
  pub position: Vector3,

  /// Ротация сущности
  pub rotation: Rotation,

  /// Скорость сущности
  pub velocity: Vector3,

  /// Состояние `on_ground` сущности
  pub on_ground: bool,
}

impl Default for Entity {
  fn default() -> Self {
    Self {
      kind: EntityKind::Null,
      uuid: Uuid::nil(),
      position: Vector3::zero(),
      rotation: Rotation::zero(),
      velocity: Vector3::zero(),
      on_ground: false,
    }
  }
}
