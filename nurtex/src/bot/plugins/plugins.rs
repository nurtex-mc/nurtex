/// Структура плагина `AutoRespawn`
#[derive(Clone)]
pub struct AutoRespawnPlugin {
  pub enabled: bool,
  pub respawn_delay: u64,
}

impl Default for AutoRespawnPlugin {
  fn default() -> Self {
    Self { enabled: true, respawn_delay: 0 }
  }
}

/// Структура плагина `AutoReconnect`
#[derive(Clone)]
pub struct AutoReconnectPlugin {
  pub enabled: bool,
  pub reconnect_delay: u64,
  pub max_attempts: i32,
}

impl Default for AutoReconnectPlugin {
  fn default() -> Self {
    Self {
      enabled: false,
      reconnect_delay: 1000,
      max_attempts: -1,
    }
  }
}
