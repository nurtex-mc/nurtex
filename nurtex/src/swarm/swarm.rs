use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::bot::Bot;
use crate::bot::handlers::Handlers;
use crate::protocol::types::BlockPos;
use crate::random::generate_username;
use crate::storage::Storage;
use crate::swarm::{JoinDelay, TargetServer};
use crate::world::Entity;

/// Рой ботов.  
///
/// Данная структура использует специальную архитектуру,
/// которая позволяет ботам хранить данные о мире и
/// обработчики событий в одном месте. Из-за этого потребление
/// RAM значительно меньше, чем при запуске тех же ботов по отдельности.
///
/// ## Примеры
///
/// ```rust, ignore
/// use nurtex::{Bot, JoinDelay, Swarm};
/// use nurtex::bot::BotChatExt;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём рой
///   let mut swarm = Swarm::create()
///     .with_join_delay(JoinDelay::fixed(500))
///     .bind("localhost", 25565);
///
///   // Добавляем ботов в рой
///   for i in 0..6 {
///     swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
///   }
///
///   // Запускаем рой
///   swarm.launch().await;
///
///   // Ждём немного
///   tokio::time::sleep(std::time::Duration::from_secs(2)).await;
///
///   // Отправляем от каждого бота сообщение в чат
///   swarm.for_each_parallel(async |bot| {
///     bot.chat_message(format!("Привет, я {}!", bot.username())).await
///   });
///
///   // Ждём немного
///   tokio::time::sleep(std::time::Duration::from_secs(5)).await;
///
///   // Выключаем рой
///   swarm.shutdown().await
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/NurtexMC/nurtex/blob/main/nurtex/examples)
pub struct Swarm {
  /// Список всех ботов
  pub bots: Vec<Arc<Bot>>,

  /// Данные целевого сервера
  target_server: Arc<RwLock<TargetServer>>,

  /// Интервал между подключениями ботов
  join_delay: Arc<JoinDelay>,

  /// Список всех хэндлов
  handles: Vec<JoinHandle<core::result::Result<(), std::io::Error>>>,

  /// Общее хранилище данных
  shared_storage: Arc<Storage>,

  /// Общие обработчики событий
  shared_handlers: Arc<Handlers>,
}

impl Swarm {
  /// Метод создания нового роя
  pub fn create() -> Self {
    Self {
      bots: Vec::new(),
      target_server: Arc::new(RwLock::new(TargetServer::default())),
      join_delay: Arc::new(JoinDelay::fixed(1000)),
      handles: Vec::new(),
      shared_storage: Arc::new(Storage::null()),
      shared_handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод создания нового роя со случайными ботами
  pub fn create_random(bots_count: usize) -> Self {
    let mut swarm = Self::create();

    for _ in 0..bots_count {
      let random_username = loop {
        let username = generate_username(rand::thread_rng().gen_range(5..=14));

        if swarm.username_is_unique(&username) {
          break username;
        }
      };

      swarm.add_bot(Bot::create(random_username));
    }

    swarm
  }

  /// Метод создания нового роя с указанием ёмкости
  pub fn create_with_capacity(capacity: usize) -> Self {
    Self {
      bots: Vec::with_capacity(capacity),
      target_server: Arc::new(RwLock::new(TargetServer::default())),
      join_delay: Arc::new(JoinDelay::fixed(1000)),
      handles: Vec::with_capacity(capacity),
      shared_storage: Arc::new(Storage::null()),
      shared_handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод установки задержки подключения
  pub fn with_join_delay(mut self, join_delay: JoinDelay) -> Self {
    self.join_delay = Arc::new(join_delay);
    self
  }

  /// Метод установки обработчиков.
  ///
  /// **Важное примечание:** Данный метод нужно вызывать строго
  /// до добавления ботов, иначе боты, добавленные в рой до
  /// использования этого метода, **не будут** использовать обработчики
  pub fn with_handlers(mut self, handlers: Handlers) -> Self {
    self.shared_handlers = Arc::new(handlers);
    self
  }

  /// Метод установки общих обработчиков
  pub fn with_shared_handlers(mut self, handlers: Arc<Handlers>) -> Self {
    self.shared_handlers = handlers;
    self
  }

  /// Метод привязки роя к адерсу целевого сервера
  pub fn bind(self, server_host: impl Into<String>, server_port: u16) -> Self {
    match self.target_server.try_write() {
      Ok(mut guard) => {
        guard.host = server_host.into();
        guard.port = server_port;
      }
      Err(_) => {}
    };

    self
  }

  /// Метод перепривязки роя к адерсу целевого сервера
  pub async fn rebind(&self, server_host: impl Into<String>, server_port: u16) {
    let mut guard = self.target_server.write().await;
    guard.host = server_host.into();
    guard.port = server_port;
  }

  /// Метод получения общего хранилища
  pub fn get_shared_storage(&self) -> Arc<Storage> {
    Arc::clone(&self.shared_storage)
  }

  /// Последовательный for-each
  pub async fn for_each_consistent<F, Fut>(&self, f: F) -> std::io::Result<()>
  where
    F: Fn(Arc<Bot>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    for bot in &self.bots {
      f(Arc::clone(bot)).await?;
    }

    Ok(())
  }

  /// Параллельный for-each
  pub fn for_each_parallel<F, Fut>(&self, f: F)
  where
    F: Fn(Arc<Bot>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.bots.iter().for_each(|bot| {
      tokio::spawn(f(Arc::clone(&bot)));
    });
  }

  /// Метод добавления бота в рой
  pub fn add_bot(&mut self, bot: Bot) {
    let swarm_bot = bot
      .set_shared_storage(Arc::clone(&self.shared_storage))
      .set_shared_handlers(Arc::clone(&self.shared_handlers));

    self.bots.push(Arc::new(swarm_bot));
  }

  /// Метод добавления нескольких ботов в рой
  pub fn add_bots(&mut self, bots: Vec<Bot>) {
    for bot in bots {
      let swarm_bot = bot
        .set_shared_storage(Arc::clone(&self.shared_storage))
        .set_shared_handlers(Arc::clone(&self.shared_handlers));

      self.bots.push(Arc::new(swarm_bot));
    }
  }

  /// Метод добавления бота в рой (возвращает `Self`)
  pub fn with_bot(mut self, bot: Bot) -> Self {
    let swarm_bot = bot
      .set_shared_storage(Arc::clone(&self.shared_storage))
      .set_shared_handlers(Arc::clone(&self.shared_handlers));

    self.bots.push(Arc::new(swarm_bot));
    self
  }

  /// Метод добавления нескольких ботов в рой (возвращает `Self`)
  pub fn with_bots(mut self, bots: Vec<Bot>) -> Self {
    for bot in bots {
      let swarm_bot = bot
        .set_shared_storage(Arc::clone(&self.shared_storage))
        .set_shared_handlers(Arc::clone(&self.shared_handlers));

      self.bots.push(Arc::new(swarm_bot));
    }

    self
  }

  /// Метод проверки уникальности юзернейма.
  /// Он сверяет данный юзернейм со всеми юзернеймами уже ранее добавленных ботов в рой
  pub fn username_is_unique(&self, username: &str) -> bool {
    for bot in &self.bots {
      if username == bot.username() {
        return false;
      }
    }

    true
  }

  /// Метод запуска роя
  pub async fn launch(&mut self) {
    let total_bots = self.bots.len();
    let (host, port) = {
      let guard = self.target_server.read().await;
      (guard.host.clone(), guard.port)
    };

    for (index, bot) in self.bots.iter().enumerate() {
      let handle = bot.connect_with_handle(&host, port);
      self.handles.push(handle);

      let delay = self.join_delay.activate(index, total_bots);

      if index < total_bots - 1 {
        tokio::time::sleep(Duration::from_millis(delay)).await;
      }
    }
  }

  /// Метод запуска роя и ожидания хэндлов
  pub async fn launch_and_wait(&mut self) {
    self.launch().await;
    self.wait_handles().await;
  }

  /// Метод мгновенного запуска роя (без задержки)
  pub fn instant_launch(&mut self) {
    let (host, port) = match self.target_server.try_read() {
      Ok(g) => (g.host.clone(), g.port),
      Err(_) => ("localhost".to_string(), 25565),
    };

    for bot in &self.bots {
      let handle = bot.connect_with_handle(&host, port);
      self.handles.push(handle);
    }
  }

  /// Метод **тихого** запуска роя (не блокирует текущий поток).
  /// Важно понимать что он **НЕ** добавляет хэндлы подключений ботов,
  /// соответственно любое взаимодействие с ними будет невозможным,
  /// так же **могут быть проблемы** при остановке роя (редко и
  /// только если выполняются долгие блокирующие операции с подключениями).
  /// В результате вызова данного метода вернётся `JoinHandle`, при
  /// помощи него можно контролировать хэндлы всех запущенных ботов
  pub fn quiet_launch(&self) -> JoinHandle<()> {
    let total_bots = self.bots.len();
    let join_delay = Arc::clone(&self.join_delay);

    let bots = {
      let mut vec = Vec::new();

      for bot in &self.bots {
        vec.push(Arc::clone(bot));
      }

      vec
    };

    let (host, port) = match self.target_server.try_read() {
      Ok(g) => (g.host.clone(), g.port),
      Err(_) => ("localhost".to_string(), 25565),
    };

    tokio::spawn(async move {
      let mut handles = Vec::with_capacity(total_bots);

      for (index, bot) in bots.iter().enumerate() {
        let handle = bot.connect_with_handle(&host, port);
        handles.push(handle);

        let delay = join_delay.activate(index, total_bots);

        if index < total_bots - 1 {
          tokio::time::sleep(Duration::from_millis(delay)).await;
        }
      }

      for handle in handles {
        let _ = handle.await;
      }
    })
  }

  /// Метод получения количества ботов в рое
  pub fn bots_count(&self) -> usize {
    self.bots.len()
  }

  /// Метод получения количества хэндлов (обычно равняется количеству ботов)
  pub fn handles_count(&self) -> usize {
    self.handles.len()
  }

  /// Метод проверки существования ботов в рое
  pub fn is_null(&self) -> bool {
    self.bots.is_empty()
  }

  /// Метод получения всех юзернеймов ботов
  pub fn get_bot_usernames(&self) -> Vec<String> {
    self.bots.iter().map(|bot| bot.username().to_string()).collect()
  }

  /// Метод выключения и очистки роя.
  /// После использования этого метода список ботов и их хэндлов полностью очищается,
  /// запустить тот же рой будет невозможно без нового добавления ботов через метод `add_bot`
  pub async fn shutdown(&mut self) -> std::io::Result<()> {
    self.abort_handles();

    tokio::time::sleep(Duration::from_millis(100)).await;

    // По сути все задачи ботов, связанные с подключением, должны уничтожиться
    // и соответственно все `NurtexConnection` должны быть доступны для записи
    for bot in &self.bots {
      bot.shutdown().await?;
    }

    self.handles.clear();
    self.bots.clear();
    self.shared_storage.clear().await;

    Ok(())
  }

  /// Метод отмены всех хэндлов, если нужно корректно и полноценно
  /// остановить рой, используй метод `shutdown`
  pub fn abort_handles(&self) {
    for handle in &self.handles {
      handle.abort();
    }
  }

  /// Метод ожидания завершения всех хэндлов
  pub async fn wait_handles(&mut self) {
    for handle in &mut self.handles {
      if !handle.is_finished() {
        // Думаю, здесь логичнее игнорировать любые ошибки
        let _ = handle.await;
      }
    }
  }

  /// Метод получения клона сущности по ID
  pub async fn get_entity(&self, id: &i32) -> Option<Entity> {
    self.shared_storage.get_entity(id).await
  }

  /// Метод получения блока по координатам
  pub async fn get_block(&self, pos: BlockPos) -> Option<u32> {
    self.shared_storage.get_block(pos).await
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use crate::bot::Bot;
  use crate::bot::handlers::Handlers;
  use crate::swarm::{JoinDelay, Swarm};

  #[tokio::test]
  async fn test_instant() -> std::io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(10);

    for i in 0..10 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.instant_launch();

    tokio::time::sleep(Duration::from_secs(3)).await;

    swarm.for_each_parallel(async |bot| {
      let position = bot.get_position().await;
      let rotation = bot.get_rotation().await;

      println!("[{}] Позиция: {:?}, Ротация: {:?}", bot.username(), position, rotation);

      Ok(())
    });

    tokio::time::sleep(Duration::from_secs(8)).await;

    swarm.shutdown().await?;

    Ok(())
  }

  #[tokio::test]
  async fn test_quiet() -> std::io::Result<()> {
    let mut bots = Vec::new();

    for i in 0..10 {
      bots.push(Bot::create(format!("nurtex_{}", i)));
    }

    let mut swarm = Swarm::create_with_capacity(10)
      .with_bots(bots)
      .with_join_delay(JoinDelay::fixed(200))
      .bind("localhost", 25565);

    let handle = swarm.quiet_launch();

    tokio::time::sleep(Duration::from_secs(1)).await;

    handle.abort();
    swarm.shutdown().await?;

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
  }

  #[tokio::test]
  async fn test_wait_handles() -> std::io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(6).with_join_delay(JoinDelay::fixed(200)).bind("localhost", 25565);

    for i in 0..6 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch().await;

    swarm.wait_handles().await;

    Ok(())
  }

  #[tokio::test]
  async fn test_shared_storage() -> std::io::Result<()> {
    let mut swarm = Swarm::create_with_capacity(6).with_join_delay(JoinDelay::fixed(200)).bind("localhost", 25565);

    for i in 0..6 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch().await;

    for _ in 0..5 {
      let storage = swarm.get_shared_storage();

      let entities = { storage.entities.read().await.clone() };

      println!("Сущности: {:?}", entities);

      tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
  }

  #[tokio::test]
  async fn test_shared_handlers() {
    let mut handlers = Handlers::new();

    handlers.on_login(async |username| {
      println!("Бот {} залогинился", username);
      Ok(())
    });

    handlers.on_spawn(async |username| {
      println!("Бот {} заспавнился", username);
      Ok(())
    });

    handlers.on_chat(async |username, payload| {
      println!("Бот {} получил сообщение: {}", username, payload.message);
      Ok(())
    });

    handlers.on_disconnect(async |username, payload| {
      println!("Бот {} отключился в состоянии: {:?}", username, payload.state);
      Ok(())
    });

    let mut swarm = Swarm::create().with_join_delay(JoinDelay::fixed(50)).with_handlers(handlers).bind("localhost", 25565);

    for i in 0..200 {
      swarm.add_bot(Bot::create(format!("nurtex_{}", i)));
    }

    swarm.launch().await;
    swarm.wait_handles().await
  }

  #[tokio::test]
  async fn test_random() {
    Swarm::create_random(10)
      .with_join_delay(JoinDelay::fixed(50))
      .bind("localhost", 25565)
      .launch_and_wait()
      .await
  }
}
