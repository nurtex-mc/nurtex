use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use hashbrown::HashMap;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::bot::connection::{BotPackage, spawn_connection};
use crate::bot::handlers::Handlers;
use crate::bot::plugins::Plugins;
use crate::bot::types::{PacketReader, PacketWriter};
use crate::bot::{BotComponents, BotProfile, ClientInfo};
use crate::protocol::connection::{ClientsidePacket, Connection};
use crate::protocol::packets::play::ServersidePlayPacket;
use crate::protocol::types::{BlockPos, Rotation, Vector3};
use crate::registry::BlockKind;
use crate::storage::Storage;
use crate::world::{Entity, EntityId};

#[cfg(feature = "proxy")]
use crate::proxy::Proxy;

#[cfg(feature = "speedometer")]
use crate::speedometer::Speedometer;

#[cfg(feature = "random")]
use crate::random::generate_username;

/// Структура Minecraft бота.
///
/// ## Примеры
/// ```rust, ignore
/// use nurtex::bot::{Bot, BotChatExt};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///   // Создаём бота
///   let mut bot = Bot::create("nurtex_bot");
///
///   // Подключаем бота к серверу
///   bot.connect("localhost", 25565);
///
///   // Ждём немножко
///   tokio::time::sleep(std::time::Duration::from_secs(3)).await;
///
///   // Отправляем сообщение в чат
///   bot.chat_message("Привет, мир!").await?;
///
///   // Ожидаем окончания хэндла подключения
///   bot.wait_handle().await
/// }
/// ```
///
/// Больше актуальных примеров: [смотреть](https://github.com/NurtexMC/nurtex/blob/main/nurtex/examples)
pub struct Bot {
  pub profile: Arc<RwLock<BotProfile>>,
  pub connection: Arc<RwLock<Connection>>,
  handle: Option<JoinHandle<core::result::Result<(), std::io::Error>>>,
  entity_id: Arc<EntityId>,
  username: String,
  protocol_version: i32,
  connection_timeout: u64,
  reader_tx: PacketReader,
  writer_tx: PacketWriter,
  #[cfg(feature = "proxy")]
  proxy: Option<Arc<Proxy>>,
  plugins: Arc<Plugins>,
  #[cfg(feature = "speedometer")]
  speedometer: Option<Arc<Speedometer>>,
  components: Arc<RwLock<BotComponents>>,
  storage: Arc<Storage>,
  handlers: Arc<Handlers>,
}

impl Bot {
  /// Метод создания нового бота
  pub fn create(username: impl Into<String>) -> Self {
    Self::create_with_options(username, 45, 45)
  }

  /// Метод создания нового бота с случайным юзернеймом
  #[cfg(feature = "random")]
  pub fn create_random() -> Self {
    use rand::Rng;
    Self::create_with_options(generate_username(rand::thread_rng().gen_range(5..=14)), 45, 45)
  }

  /// Метод создания нового бота с прокси
  #[cfg(feature = "proxy")]
  pub fn create_with_proxy(username: impl Into<String>, proxy: Proxy) -> Self {
    Self::create_with_options(username, 45, 45).with_proxy(proxy)
  }

  /// Метод создания нового бота со спидометром
  #[cfg(feature = "speedometer")]
  pub fn create_with_speedometer(username: impl Into<String>, speedometer: Arc<Speedometer>) -> Self {
    Self::create_with_options(username, 45, 45).with_speedometer(speedometer)
  }

  /// Метод создания нового бота с заданными опциями
  pub fn create_with_options(username: impl Into<String>, reader_capacity: usize, writer_capacity: usize) -> Self {
    let (reader_tx, _) = broadcast::channel(reader_capacity);
    let (writer_tx, _) = broadcast::channel(writer_capacity);

    let name = username.into();
    let profile = BotProfile::new(name.clone());

    Self {
      profile: Arc::new(RwLock::new(profile)),
      connection: Arc::new(RwLock::new(Connection::new())),
      plugins: Arc::new(Plugins::default()),
      protocol_version: 774,
      connection_timeout: 14000,
      #[cfg(feature = "proxy")]
      proxy: None,
      entity_id: Arc::new(EntityId::negative()),
      username: name,
      handle: None,
      reader_tx: Arc::new(reader_tx),
      writer_tx: Arc::new(writer_tx),
      #[cfg(feature = "speedometer")]
      speedometer: None,
      components: Arc::new(RwLock::new(BotComponents::default())),
      storage: Arc::new(Storage::null()),
      handlers: Arc::new(Handlers::new()),
    }
  }

  /// Метод запуска `reader` (выполняется автоматически при подключении бота)
  pub fn run_reader(connection: Arc<RwLock<Connection>>, reader_tx: PacketReader) -> JoinHandle<()> {
    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_millis(500)).await;

      loop {
        let packet_result = {
          match tokio::time::timeout(Duration::from_secs(14), connection.read()).await {
            Ok(g) => g.read_packet().await,
            _ => None,
          }
        };

        match packet_result {
          Some(packet) => {
            if reader_tx.send(packet).is_err() {
              break;
            }
          }
          None => tokio::time::sleep(Duration::from_millis(50)).await,
        }
      }
    })
  }

  /// Метод запуска `writer` (выполняется автоматически при подключении бота)
  pub fn run_writer(connection: Arc<RwLock<Connection>>, writer_tx: PacketWriter) -> JoinHandle<()> {
    let mut writer_rx = writer_tx.subscribe();

    tokio::spawn(async move {
      tokio::time::sleep(Duration::from_millis(800)).await;

      let writer_fn = async |packet: ServersidePlayPacket| {
        let conn_guard = connection.read().await;

        if let Err(_) = conn_guard.write_play_packet(packet).await {
          tokio::time::sleep(Duration::from_millis(100)).await
        }
      };

      loop {
        if let Ok(packet) = writer_rx.recv().await {
          match tokio::time::timeout(Duration::from_secs(14), writer_fn(packet)).await {
            Ok(_) => continue,
            Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
          }
        }
      }
    })
  }

  /// Метод установки плагинов
  pub fn with_plugins(mut self, plugins: Plugins) -> Self {
    self.plugins = Arc::new(plugins);
    self
  }

  /// Метод установки спидометра
  #[cfg(feature = "speedometer")]
  pub fn with_speedometer(mut self, speedometer: Arc<Speedometer>) -> Self {
    self.speedometer = Some(speedometer);
    self
  }

  /// Метод установки версии протокола
  pub fn with_protocol_version(mut self, protocol_version: i32) -> Self {
    self.protocol_version = protocol_version;
    self
  }

  /// Метод установки таймаута подключения
  pub fn with_connection_timeout(mut self, timeout: u64) -> Self {
    self.connection_timeout = timeout;
    self
  }

  /// Метод установки прокси
  #[cfg(feature = "proxy")]
  pub fn with_proxy(mut self, proxy: Proxy) -> Self {
    self.proxy = Some(Arc::new(proxy));
    self
  }

  /// Метод установки информации клиента
  pub fn with_information(self, information: ClientInfo) -> Self {
    // Здесь почти невозможен исход с ошибкой, поэтому просто игнорируем
    match self.profile.try_write() {
      Ok(mut g) => g.information = information,
      Err(_) => {}
    }

    self
  }

  /// Метод установки обработчиков
  pub fn with_handlers(mut self, handlers: Handlers) -> Self {
    self.handlers = Arc::new(handlers);
    self
  }

  /// Метод установки общего хранилища
  pub fn set_shared_storage(mut self, storage: Arc<Storage>) -> Self {
    self.storage = storage;
    self
  }

  /// Метод установки общих обработчиков
  pub fn set_shared_handlers(mut self, handlers: Arc<Handlers>) -> Self {
    self.handlers = handlers;
    self
  }

  /// Метод получения юзернейма
  pub fn username(&self) -> &str {
    &self.username
  }

  /// Метод получения UUID
  pub async fn uuid(&self) -> Uuid {
    self.profile.read().await.uuid
  }

  /// Метод получения профиля бота
  pub fn get_profile(&self) -> Arc<RwLock<BotProfile>> {
    Arc::clone(&self.profile)
  }

  /// Метод получения прокси бота
  #[cfg(feature = "proxy")]
  pub fn get_proxy(&self) -> Option<Arc<Proxy>> {
    if let Some(proxy) = &self.proxy { Some(Arc::clone(&proxy)) } else { None }
  }

  /// Метод получения хранилища
  pub fn get_storage(&self) -> Arc<Storage> {
    Arc::clone(&self.storage)
  }

  /// Вспомогательный метод подписки на слушание пакетов
  pub fn subscribe_to_reader(&self) -> broadcast::Receiver<ClientsidePacket> {
    self.reader_tx.subscribe()
  }

  /// Метод получения копии `reader_tx`
  pub fn get_reader(&self) -> PacketReader {
    Arc::clone(&self.reader_tx)
  }

  /// Метод получения копии `writer_tx`
  pub fn get_writer(&self) -> PacketWriter {
    Arc::clone(&self.writer_tx)
  }

  /// Метод получения хэндла
  pub fn get_handle(&self) -> &Option<JoinHandle<core::result::Result<(), std::io::Error>>> {
    &self.handle
  }

  /// Метод отправки пакета
  pub fn send_packet(&self, packet: ServersidePlayPacket) {
    let _ = self.writer_tx.send(packet);
  }

  /// Метод подключения бота к серверу.
  ///
  /// ## Примеры
  ///
  /// ```rust, ignore
  /// use nurtex::Bot;
  ///
  /// #[tokio::main]
  /// async fn main() -> std::io::Result<()> {
  ///   // Создаём бота
  ///   let mut bot = Bot::create("nurtex_bot");
  ///
  ///   // Подключаем бота к серверу.
  ///   // Если после вызова этого метода выполняются
  ///   // какие-либо действия с ботом, рекомендуется
  ///   // подождать несколько секунд, чтобы бот
  ///   // полностью подключился к серверу
  ///   bot.connect("localhost", 25565);
  ///
  ///   // Ожидаем окончания хэндла подключения
  ///   bot.wait_handle().await
  /// }
  /// ```
  pub fn connect(&mut self, server_host: impl Into<String>, server_port: u16) {
    self.handle = Some(self.connect_with_handle(server_host, server_port));
  }

  /// Метод подключения бота к серверу, возвращающий хэндл подключения
  pub fn connect_with_handle(&self, server_host: impl Into<String>, server_port: u16) -> JoinHandle<Result<(), std::io::Error>> {
    let package = BotPackage {
      connection: self.connection.clone(),
      profile: self.profile.clone(),
      components: self.components.clone(),
      entity_id: self.entity_id.clone(),
      #[cfg(feature = "speedometer")]
      speedometer: self.speedometer.clone(),
      plugins: self.plugins.clone(),
      packet_reader: self.reader_tx.clone(),
      packet_writer: self.writer_tx.clone(),
      storage: self.storage.clone(),
      #[cfg(feature = "proxy")]
      proxy: self.proxy.clone(),
      handlers: self.handlers.clone(),
      server_host: server_host.into(),
      server_port: server_port,
      protocol_version: self.protocol_version,
      connection_timeout: self.connection_timeout,
    };

    tokio::spawn(async move {
      let mut reconnection_attempts = 0;
      let max_attempts = if package.plugins.auto_reconnect.enabled {
        package.plugins.auto_reconnect.max_attempts
      } else {
        1
      };

      loop {
        let reader_handle = Self::run_reader(Arc::clone(&package.connection), Arc::clone(&package.packet_reader));
        let writer_handle = Self::run_writer(Arc::clone(&package.connection), Arc::clone(&package.packet_writer));

        let result = spawn_connection(&package).await;

        // На этом моменте бот считается не подключенным к серверу, поэтому нужно отменять reader / writer
        reader_handle.abort();
        writer_handle.abort();

        match result {
          Ok(_) => return Ok(()),
          Err(e) => match e.kind() {
            ErrorKind::ConnectionAborted | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset | ErrorKind::TimedOut | ErrorKind::NotConnected | ErrorKind::NetworkDown => {
              if !package.plugins.auto_reconnect.enabled || (max_attempts != -1 && reconnection_attempts >= max_attempts) {
                return Err(e);
              }

              reconnection_attempts += 1;

              tokio::time::sleep(Duration::from_millis(package.plugins.auto_reconnect.reconnect_delay)).await;
            }
            _ => return Err(e),
          },
        }
      }
    })
  }

  /// Метод ожидания завершения хэндла подключения
  pub async fn wait_handle(&mut self) -> std::io::Result<()> {
    if let Some(handle) = self.handle.as_mut() { handle.await? } else { Ok(()) }
  }

  /// Метод полноценной очистки и отключения бота
  pub async fn shutdown(&self) -> std::io::Result<()> {
    self.abort_handle();

    let conn_guard = self.connection.read().await;
    conn_guard.shutdown().await?;

    std::mem::drop(conn_guard);

    self.clear().await;

    Ok(())
  }

  /// Метод очистки данных бота
  pub async fn clear(&self) {
    *self.components.write().await = BotComponents::default();
  }

  /// Метод отмены хэндла бота
  pub fn abort_handle(&self) {
    if let Some(handle) = &self.handle {
      handle.abort();
    }
  }

  /// Метод переподключения бота
  pub async fn reconnect(&mut self, server_host: impl Into<String>, server_port: u16, reconnect_delay: u64) -> std::io::Result<()> {
    self.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
    self.connect(server_host, server_port);
    Ok(())
  }

  /// Метод переподключения бота, возвращающий хэндл подключения
  pub async fn reconnect_with_handle(&mut self, server_host: impl Into<String>, server_port: u16, reconnect_delay: u64) -> std::io::Result<JoinHandle<Result<(), std::io::Error>>> {
    self.shutdown().await?;
    tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
    Ok(self.connect_with_handle(server_host, server_port))
  }

  /// Метод получения компонентов бота
  pub fn get_components(&self) -> Arc<RwLock<BotComponents>> {
    Arc::clone(&self.components)
  }

  /// Метод получения опциональной позиции бота
  pub fn try_get_position(&self) -> Option<Vector3> {
    match self.components.try_read() {
      Ok(g) => Some(g.position.clone()),
      Err(_) => None,
    }
  }

  /// Метод получения опционального здоровья бота
  pub fn try_get_health(&self) -> Option<Vector3> {
    match self.components.try_read() {
      Ok(g) => Some(g.position.clone()),
      Err(_) => None,
    }
  }

  /// Метод получения опциональной ротации бота
  pub fn try_get_rotation(&self) -> Option<f32> {
    match self.components.try_read() {
      Ok(g) => Some(g.health),
      Err(_) => None,
    }
  }

  /// Метод получения опциональных сущностей из хранилища
  pub fn try_get_entities(&self) -> Option<HashMap<i32, Entity>> {
    match self.storage.entities.try_read() {
      Ok(g) => {
        let mut entities = HashMap::with_capacity(g.len());

        for (entity_id, entity) in &*g {
          entities.insert(*entity_id, entity.clone());
        }

        Some(entities)
      }
      Err(_) => None,
    }
  }

  /// Метод получения ID сущности бота
  pub async fn get_entity_id(&self) -> i32 {
    self.entity_id.get()
  }

  /// Метод получения позиции бота
  pub async fn get_position(&self) -> Vector3 {
    let guard = self.components.read().await;
    guard.position.clone()
  }

  /// Метод получения ротации бота
  pub async fn get_rotation(&self) -> Rotation {
    let guard = self.components.read().await;
    guard.rotation.clone()
  }

  /// Метод получения здоровья бота
  pub async fn get_health(&self) -> f32 {
    let guard = self.components.read().await;
    guard.health
  }

  /// Метод получения всех сущностей из хранилища
  pub async fn get_entities(&self) -> HashMap<i32, Entity> {
    let guard = self.storage.entities.read().await;
    let mut entities = HashMap::with_capacity(guard.len());

    for (entity_id, entity) in &*guard {
      entities.insert(*entity_id, entity.clone());
    }

    entities
  }

  /// Метод получения блока по координатам
  pub async fn get_block(&self, pos: BlockPos) -> Option<BlockKind> {
    self.storage.get_block(pos).await
  }
}

#[cfg(test)]
mod tests {
  use std::io;
  use std::time::Duration;

  use crate::bot::handlers::Handlers;
  use crate::protocol::connection::ClientsidePacket;
  use crate::protocol::packets::play::ClientsidePlayPacket;
  use crate::protocol::types::BlockPos;
  use crate::proxy::Proxy;

  use crate::bot::plugins::{AutoReconnectPlugin, AutoRespawnPlugin, Plugins};
  use crate::bot::{Bot, BotChatExt};

  #[tokio::test]
  async fn test_packet_handling() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);

        // + Доп проверка взаимодействия с чатом

        match packet {
          ClientsidePlayPacket::KeepAlive(p) => {
            bot.chat_message(format!("Получен KeepAlive: {}", p.id)).await?;
          }
          _ => {}
        }
      }
    }
  }

  #[tokio::test]
  async fn test_handlers() -> io::Result<()> {
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

    let mut bot = Bot::create("nurtex_bot").with_handlers(handlers);

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }

  #[tokio::test]
  async fn test_auto_respawn() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").with_plugins(Plugins {
      auto_respawn: AutoRespawnPlugin {
        enabled: true,
        respawn_delay: 2000,
      },
      ..Default::default()
    });

    bot.connect("localhost", 25565);
    bot.wait_handle().await
  }

  #[tokio::test]
  async fn test_auto_reconnect() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot").with_plugins(Plugins {
      auto_reconnect: AutoReconnectPlugin {
        enabled: true,
        reconnect_delay: 1000,
        max_attempts: 3,
      },
      ..Default::default()
    });

    bot.connect("localhost", 25565);

    // + Доп проверка на работоспособность reader'а пакетов после переподключения

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_entity_storage() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    tokio::time::sleep(Duration::from_secs(3)).await;

    for _ in 0..10 {
      for (_, entity) in bot.get_entities().await {
        println!("Сущность: {:?}", entity);
      }

      tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
  }

  #[tokio::test]
  async fn test_chunk_storage() -> io::Result<()> {
    let mut bot = Bot::create("nurtex_bot");

    bot.connect("localhost", 25565);

    tokio::time::sleep(Duration::from_secs(3)).await;

    let pos = bot.get_position().await;
    let feet_block = bot
      .get_block(BlockPos {
        x: pos.x as i32,
        y: (pos.y - 1.0) as i32,
        z: pos.z as i32,
      })
      .await;

    if let Some(block) = feet_block {
      println!("Блок под ногами: {:?}", block);
    }

    Ok(())
  }

  #[tokio::test]
  async fn test_bot_with_socks5_proxy() -> io::Result<()> {
    let proxy = Proxy::from("socks5://212.58.132.5:1080");
    let mut bot = Bot::create_with_proxy("l7jqw8d5", proxy);

    bot.connect("hub.holyworld.ru", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_bot_with_socks4_proxy() -> io::Result<()> {
    let proxy = Proxy::from("socks4://68.71.242.118:4145");
    let mut bot = Bot::create_with_proxy("k72ido3d", proxy);

    bot.connect("hub.holyworld.ru", 25565);

    let mut reader = bot.subscribe_to_reader();

    loop {
      if let Ok(ClientsidePacket::Play(packet)) = reader.recv().await {
        println!("Бот {} получил пакет: {:?}", bot.username(), packet);
      }
    }
  }

  #[tokio::test]
  async fn test_reconnect() -> io::Result<()> {
    let mut handlers = Handlers::new();

    handlers.on_spawn(async |username| {
      println!("Бот {} заспавнился", username);
      Ok(())
    });

    let mut bot = Bot::create("nurtex_bot").with_handlers(handlers);

    let server_host = "localhost".to_string();
    let server_port = 25565;

    bot.connect(&server_host, server_port);

    tokio::time::sleep(Duration::from_secs(3)).await;

    bot.reconnect(&server_host, server_port, 1000).await?;

    bot.wait_handle().await
  }
}
