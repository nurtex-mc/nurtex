use nurtex_protocol::connection::ConnectionState;
use uuid::Uuid;

/// Данные обработчика события `chat`
pub struct ChatPayload {
  pub message: String,
  pub sender_uuid: Uuid,
}

/// Данные обработчика события `disconnect`
pub struct DisconnectPayload {
  pub state: ConnectionState,
}
