/// Данные целевого сервера
pub struct TargetServer {
  pub host: String,
  pub port: u16,
}

impl Default for TargetServer {
  fn default() -> Self {
    Self {
      host: "localhost".to_string(),
      port: 25565,
    }
  }
}
