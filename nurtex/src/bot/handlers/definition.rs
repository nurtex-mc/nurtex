use std::sync::Arc;

use crate::bot::handlers::payloads::*;

/// Вспомогательный тип результата функции обработчика
pub type HandlerResult = Box<dyn std::future::Future<Output = std::io::Result<()>> + Send>;

/// Структура обработчиков бота
pub struct Handlers {
  pub on_login_handler: Option<Arc<dyn Fn(String) -> std::pin::Pin<HandlerResult> + Send + Sync>>,
  pub on_spawn_handler: Option<Arc<dyn Fn(String) -> std::pin::Pin<HandlerResult> + Send + Sync>>,
  pub on_chat_handler: Option<Arc<dyn Fn(String, ChatPayload) -> std::pin::Pin<HandlerResult> + Send + Sync>>,
  pub on_disconnect_handler: Option<Arc<dyn Fn(String, DisconnectPayload) -> std::pin::Pin<HandlerResult> + Send + Sync>>,
}

impl Handlers {
  /// Метод создания новых обработчиков
  pub fn new() -> Self {
    Self {
      on_login_handler: None,
      on_spawn_handler: None,
      on_chat_handler: None,
      on_disconnect_handler: None,
    }
  }

  /// Метод установки обработчика на событие `login`
  pub fn on_login<F, O>(&mut self, handler: F)
  where
    F: Fn(String) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.on_login_handler = Some(Arc::new(move |username| Box::pin(handler(username))));
  }

  /// Метод установки обработчика на событие `spawn`
  pub fn on_spawn<F, O>(&mut self, handler: F)
  where
    F: Fn(String) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.on_spawn_handler = Some(Arc::new(move |username| Box::pin(handler(username))));
  }

  /// Метод установки обработчика на событие `chat`
  pub fn on_chat<F, O>(&mut self, handler: F)
  where
    F: Fn(String, ChatPayload) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.on_chat_handler = Some(Arc::new(move |username, payload| Box::pin(handler(username, payload))));
  }

  /// Метод установки обработчика на событие `disconnect`
  pub fn on_disconnect<F, O>(&mut self, handler: F)
  where
    F: Fn(String, DisconnectPayload) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.on_disconnect_handler = Some(Arc::new(move |username, payload| Box::pin(handler(username, payload))));
  }
}
