use std::sync::Arc;

use tokio::sync::RwLock;

use crate::bot::BotComponents;

/// Функция временного захвата компонентов
pub async fn capture_components<F>(components: &Arc<RwLock<BotComponents>>, f: F)
where
  F: AsyncFnOnce(&mut BotComponents),
{
  let mut guard = components.write().await;
  f(&mut *guard).await
}
