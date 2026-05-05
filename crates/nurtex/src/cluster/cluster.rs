use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::{Bot, JoinDelay, Swarm, bot::handlers::Handlers};

/// Кластер роев (или же скопление роев).
///
/// Кластер позволяет запускать рои одновременно
/// на нескольких различных серверах (мульти-таргет)
/// и удобно управлять всеми роями / ботами в роях.
///
/// Кластер имеет архитектуру, похожую на архитектуру роя,
/// но отличается тем, что **не хранит** данные о мире
/// в едином месте, так как из-за того что на разных
/// серверах разные миры - хранение общих данных о мире
/// не будет иметь какого-то смысла и только усложнит работу.
///
/// ## Примеры
///
/// ```rust, ignore
/// use nurtex::{Bot, Cluster, JoinDelay};
/// use nurtex::bot::BotChatExt;
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём кластер
///   let mut cluster = Cluster::create();
///
///   for s_ind in 0..3 {
///     // Создаём список ботов
///     let mut bots = Vec::new();
///
///     for b_ind in 0..2 {
///       // Создаём бота и добавляем его в список
///       bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
///     }
///
///     // Добавляем рой в кластер
///     cluster.add_swarm(bots, JoinDelay::fixed(1000), "localhost", 25565);
///   }
///
///   // Запускаем кластер
///   cluster.launch();
///
///   // Ждём немножко
///   tokio::time::sleep(std::time::Duration::from_secs(5)).await;
///
///   // Проходимся параллельно по всем ботам из всех роев
///   cluster.for_each_bots_parallel(async |bot| {
///     // Отправляем сообщение в чат
///     bot.chat_message(format!("Привет, я {}!", bot.username())).await
///   });
///
///   // Ожидаем заврещения всех хэндлов
///   cluster.wait_finish().await
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/NurtexMC/nurtex/blob/main/crates/nurtex/examples)
pub struct Cluster {
  /// Список роев
  swarms: Vec<Arc<Swarm>>,

  /// Список хэндлов
  handles: Vec<JoinHandle<()>>,

  /// Общие обработчики событий
  shared_handlers: Arc<Handlers>,
}

impl Cluster {
  /// Метод создания нового скопления
  pub fn create() -> Self {
    Self {
      swarms: Vec::new(),
      handles: Vec::new(),
      shared_handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод создания нового скопление с заданной ёмкостью
  pub fn create_with_capacity(capacity: usize) -> Self {
    Self {
      swarms: Vec::with_capacity(capacity),
      handles: Vec::with_capacity(capacity),
      shared_handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод установки обработчиков.
  ///
  /// **Важное примечание:** Данный метод нужно вызывать строго
  /// до добавления роев, иначе рои, добавленные в кластер до
  /// использования этого метода, **не будут** использовать обработчики
  pub fn with_handlers(mut self, handlers: Handlers) -> Self {
    self.shared_handlers = Arc::new(handlers);
    self
  }

  /// Метод добавления роя
  pub fn add_swarm(&mut self, bots: Vec<Bot>, join_delay: JoinDelay, target_host: impl Into<String>, target_port: u16) {
    let mut swarm = Swarm::create()
      .with_join_delay(join_delay)
      .with_shared_handlers(Arc::clone(&self.shared_handlers))
      .bind(target_host, target_port);

    for bot in bots {
      swarm.add_bot(bot);
    }

    self.swarms.push(Arc::new(swarm));
  }

  /// Метод добавления роя (возвращает `Self`)
  pub fn with_swarm(mut self, bots: Vec<Bot>, join_delay: JoinDelay, target_host: impl Into<String>, target_port: u16) -> Self {
    let mut swarm = Swarm::create()
      .with_join_delay(join_delay)
      .with_shared_handlers(Arc::clone(&self.shared_handlers))
      .bind(target_host, target_port);

    for bot in bots {
      swarm.add_bot(bot);
    }

    self.swarms.push(Arc::new(swarm));

    self
  }

  /// Метод получения роя по его индексу
  pub fn get_swarm(&self, swarm_id: usize) -> Option<Arc<Swarm>> {
    if let Some(swarm) = self.swarms.get(swarm_id) { Some(Arc::clone(swarm)) } else { None }
  }

  /// Метод получения всех роев
  pub fn get_all_swarms(&self) -> Vec<Arc<Swarm>> {
    let mut swarms = Vec::with_capacity(self.swarms.len());

    for swarm in &self.swarms {
      swarms.push(Arc::clone(swarm));
    }

    swarms
  }

  /// Метод запуска кластера
  pub fn launch(&mut self) {
    for swarm in &self.swarms {
      self.handles.push(swarm.quiet_launch());
    }
  }

  /// Метод запуска кластера и ожидания завершения хэндлов
  pub async fn launch_and_wait(&mut self) -> std::io::Result<()> {
    for swarm in &self.swarms {
      self.handles.push(swarm.quiet_launch());
    }

    self.wait_finish().await
  }

  /// Метод запуска опредлённого роя из кластера
  pub fn launch_swarm(&mut self, swarm_id: i32) {
    for (id, swarm) in self.swarms.iter().enumerate() {
      if id as i32 != swarm_id {
        continue;
      }

      self.handles.push(swarm.quiet_launch());
    }
  }

  /// Последовательный `for-each` по всем роям
  pub async fn for_each_consistent<F, O>(&self, f: F) -> std::io::Result<()>
  where
    F: Fn(Arc<Swarm>) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    for swarm in &self.swarms {
      f(Arc::clone(swarm)).await?;
    }

    Ok(())
  }

  /// Параллельный `for-each` по всем роям
  pub fn for_each_parallel<F, O>(&self, f: F)
  where
    F: Fn(Arc<Swarm>) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.swarms.iter().for_each(|swarm| {
      tokio::spawn(f(Arc::clone(&swarm)));
    });
  }

  /// Последовательный `for-each` по всем ботам
  pub async fn for_each_bots_consistent<F, O>(&self, f: F) -> std::io::Result<()>
  where
    F: Fn(Arc<Bot>) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    let f = Arc::new(f);

    for swarm in &self.swarms {
      for bot in &swarm.bots {
        f(Arc::clone(bot)).await?;
      }
    }

    Ok(())
  }

  /// Параллельный `for-each` по всем ботам
  pub fn for_each_bots_parallel<F, O>(&self, f: F)
  where
    F: Fn(Arc<Bot>) -> O + Send + Sync + 'static,
    O: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
  {
    self.swarms.iter().for_each(|swarm| {
      swarm.bots.iter().for_each(|bot| {
        tokio::spawn(f(Arc::clone(&bot)));
      });
    });
  }

  /// Метод ожидания завершения всех хэндлов
  pub async fn wait_finish(&mut self) -> std::io::Result<()> {
    for handle in &mut self.handles {
      handle.await?;
    }

    Ok(())
  }

  /// Метод полной очистки и остановки кластера
  pub async fn shutdown(&mut self) {
    self.abort_handles();
    self.swarms.clear();
    self.handles.clear();
  }

  /// Метод отмены хэндлов
  pub fn abort_handles(&self) {
    for handle in &self.handles {
      if !handle.is_finished() {
        handle.abort();
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use crate::{
    Bot, Cluster, JoinDelay,
    bot::{BotChatExt, handlers::Handlers},
  };

  #[tokio::test]
  async fn test_minimal_cluster() -> std::io::Result<()> {
    let mut cluster = Cluster::create();

    for si in 0..3 {
      let mut bots = Vec::new();

      for bi in 0..2 {
        bots.push(Bot::create(format!("nurtex_{}_{}", si, bi)));
      }

      cluster.add_swarm(bots, JoinDelay::fixed(5000), "localhost", 25565);
    }

    cluster.launch();

    tokio::time::sleep(Duration::from_secs(5)).await;

    cluster.wait_finish().await
  }

  #[tokio::test]
  async fn test_for_each_bots() -> std::io::Result<()> {
    let mut cluster = Cluster::create();

    for si in 0..3 {
      let mut bots = Vec::new();

      for bi in 0..2 {
        bots.push(Bot::create(format!("nurtex_{}_{}", si, bi)));
      }

      cluster.add_swarm(bots, JoinDelay::fixed(5000), "localhost", 25565);
    }

    cluster.launch();

    tokio::time::sleep(Duration::from_secs(6)).await;

    cluster.for_each_bots_parallel(async |bot| bot.chat_message("Параллельный for-each").await);

    tokio::time::sleep(Duration::from_secs(2)).await;

    cluster
      .for_each_bots_consistent(async |bot| {
        tokio::time::sleep(Duration::from_millis(250)).await;
        bot.chat_message("Последовательный for-each").await
      })
      .await?;

    cluster.wait_finish().await
  }

  #[tokio::test]
  async fn test_shared_handlers() -> std::io::Result<()> {
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

    let mut cluster = Cluster::create().with_handlers(handlers);

    for si in 0..3 {
      let mut bots = Vec::new();

      for bi in 0..10 {
        bots.push(Bot::create(format!("nurtex_{}_{}", si, bi)));
      }

      cluster.add_swarm(bots, JoinDelay::fixed(5000), "localhost", 25565);
    }

    cluster.launch_and_wait().await
  }
}
