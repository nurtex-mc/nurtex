use std::sync::Arc;

use tokio::sync::RwLock;

use crate::bot::BotComponents;
use crate::protocol::connection::Connection;

/// Функция временного захвата подключения
pub async fn capture_connection<F>(connection: &Arc<RwLock<Connection>>, f: F) -> std::io::Result<()>
where
  F: AsyncFnOnce(&Connection) -> std::io::Result<()>,
{
  let conn_guard = connection.read().await;
  f(&conn_guard).await
}

/// Функция временного захвата компонентов
pub async fn capture_components<F>(components: &Arc<RwLock<BotComponents>>, f: F)
where
  F: AsyncFnOnce(&mut BotComponents),
{
  let mut guard = components.write().await;
  f(&mut *guard).await
}
