use crate::bot::plugins::{AutoReconnectPlugin, AutoRespawnPlugin};

/// Структура плагинов бота
#[derive(Clone)]
pub struct Plugins {
  pub auto_respawn: AutoRespawnPlugin,
  pub auto_reconnect: AutoReconnectPlugin,
}

impl Default for Plugins {
  fn default() -> Self {
    Self {
      auto_respawn: AutoRespawnPlugin::default(),
      auto_reconnect: AutoReconnectPlugin::default(),
    }
  }
}
