use std::fmt::Debug;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicI32, Ordering};

use bytes::BytesMut;
use nurtex_proxy::{Proxy, ProxyResult};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::connection::reader::ConnectionReader;
use crate::connection::writer::ConnectionWriter;
use crate::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideConfigurationPacket},
  handshake::{ClientsideHandshakePacket, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginPacket},
  play::{ClientsidePlayPacket, ServersidePlayPacket},
  status::{ClientsideStatusPacket, ServersideStatusPacket},
};

/// Состояние подключения
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
  Handshake,
  Status,
  Login,
  Configuration,
  Play,
}

impl From<i8> for ConnectionState {
  fn from(value: i8) -> Self {
    match value {
      -1 => Self::Status,
      0 => Self::Handshake,
      1 => Self::Login,
      2 => Self::Configuration,
      3 => Self::Play,
      _ => Self::Handshake,
    }
  }
}

/// Универсальное перечисление `Clientside` пакетов
#[derive(Debug, Clone)]
pub enum ClientsidePacket {
  Handshake(ClientsideHandshakePacket),
  Status(ClientsideStatusPacket),
  Login(ClientsideLoginPacket),
  Configuration(ClientsideConfigurationPacket),
  Play(ClientsidePlayPacket),
}

/// Универсальное перечисление `Serverside` пакетов
#[derive(Debug, Clone)]
pub enum ServersidePacket {
  Handshake(ServersideHandshakePacket),
  Status(ServersideStatusPacket),
  Login(ServersideLoginPacket),
  Configuration(ServersideConfigurationPacket),
  Play(ServersidePlayPacket),
}

impl ServersidePacket {
  /// Вспомогательный метод создания `handshake` пакета
  pub fn handshake(packet: ServersideHandshakePacket) -> Self {
    ServersidePacket::Handshake(packet)
  }

  /// Вспомогательный метод создания `status` пакета
  pub fn status(packet: ServersideStatusPacket) -> Self {
    ServersidePacket::Status(packet)
  }

  //// Вспомогательный метод создания `login` пакета
  pub fn login(packet: ServersideLoginPacket) -> Self {
    ServersidePacket::Login(packet)
  }

  /// Вспомогательный метод создания `configuration` пакета
  pub fn configuration(packet: ServersideConfigurationPacket) -> Self {
    ServersidePacket::Configuration(packet)
  }

  /// Вспомогательный метод создания `play` пакета
  pub fn play(packet: ServersidePlayPacket) -> Self {
    ServersidePacket::Play(packet)
  }
}

/// Основная структура подключения
pub struct Connection {
  /// Чтение пакетов
  reader: Arc<Mutex<ConnectionReader>>,

  /// Запись пакетов
  writer: Arc<Mutex<ConnectionWriter>>,

  /// Состояние подключения
  state: Arc<AtomicI8>,

  /// Порог сжатия (от 0 до 1024), изначально -1
  compression_threshold: Arc<AtomicI32>,
}

impl Connection {
  /// Метод создания нового подключения
  pub fn new() -> Self {
    let state = Arc::new(AtomicI8::new(0)); // Изначально хэндшейк
    let compression_threshold = Arc::new(AtomicI32::new(-1)); // Изначально отрицательный (не указан)

    let reader = ConnectionReader {
      read_stream: None,
      buffer: BytesMut::with_capacity(64 * 1024),
      compression_threshold: Arc::clone(&compression_threshold),
      decryptor: Arc::new(Mutex::new(None)),
      state: Arc::clone(&state),
    };

    let writer = ConnectionWriter {
      write_stream: None,
      compression_threshold: Arc::clone(&compression_threshold),
      encryptor: Arc::new(Mutex::new(None)),
    };

    Self {
      reader: Arc::new(Mutex::new(reader)),
      writer: Arc::new(Mutex::new(writer)),
      state: state,
      compression_threshold,
    }
  }

  /// Метод создания нового подключения
  pub async fn connect(&self, server_host: impl Into<String>, server_port: u16) -> std::io::Result<()> {
    let stream = TcpStream::connect(format!("{}:{}", server_host.into(), server_port)).await?;
    stream.set_nodelay(true)?;

    let (read_stream, write_stream) = stream.into_split();

    self.reader.lock().await.read_stream = Some(read_stream);
    self.writer.lock().await.write_stream = Some(write_stream);

    Ok(())
  }

  /// Метод создания нового подключения с прокси
  pub async fn connect_with_proxy(&self, server_host: impl Into<String>, server_port: u16, proxy: &Proxy) -> std::io::Result<()> {
    let stream = match proxy.connect(server_host, server_port).await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    stream.set_nodelay(true)?;

    let (read_stream, write_stream) = stream.into_split();

    self.reader.lock().await.read_stream = Some(read_stream);
    self.writer.lock().await.write_stream = Some(write_stream);

    Ok(())
  }

  /// Метод получения `reader`
  pub fn get_reader(&self) -> Arc<Mutex<ConnectionReader>> {
    self.reader.clone()
  }

  /// Метод получения `writer`
  pub fn get_writer(&self) -> Arc<Mutex<ConnectionWriter>> {
    self.writer.clone()
  }

  /// Метод получения текущего состояния подключения
  pub async fn get_state(&self) -> ConnectionState {
    ConnectionState::from(self.state.load(Ordering::SeqCst))
  }

  /// Метод изменения состояния подключения
  pub async fn set_state(&self, state: ConnectionState) {
    let state_id = match state {
      ConnectionState::Status => -1,
      ConnectionState::Handshake => 0,
      ConnectionState::Login => 1,
      ConnectionState::Configuration => 2,
      ConnectionState::Play => 3,
    };

    self.state.store(state_id, Ordering::SeqCst);
  }

  /// Вспомогательный метод чтения пакета
  pub async fn read_packet(&self) -> Option<ClientsidePacket> {
    let mut reader = self.reader.lock().await;
    reader.read_packet().await
  }

  /// Вспомогательный метод чтения `status` пакета
  pub async fn read_status_packet(&self) -> Option<ClientsideStatusPacket> {
    let mut reader = self.reader.lock().await;

    if let Some(ClientsidePacket::Status(packet)) = reader.read_packet().await {
      Some(packet)
    } else {
      None
    }
  }

  /// Вспомогательный метод чтения `login` пакета
  pub async fn read_login_packet(&self) -> Option<ClientsideLoginPacket> {
    let mut reader = self.reader.lock().await;

    if let Some(ClientsidePacket::Login(packet)) = reader.read_packet().await {
      Some(packet)
    } else {
      None
    }
  }

  /// Вспомогательный метод чтения `configuration` пакета
  pub async fn read_configuration_packet(&self) -> Option<ClientsideConfigurationPacket> {
    let mut reader = self.reader.lock().await;

    if let Some(ClientsidePacket::Configuration(packet)) = reader.read_packet().await {
      Some(packet)
    } else {
      None
    }
  }

  /// Вспомогательный метод чтения `play` пакета
  pub async fn read_play_packet(&self) -> Option<ClientsidePlayPacket> {
    let mut reader = self.reader.lock().await;

    if let Some(ClientsidePacket::Play(packet)) = reader.read_packet().await {
      Some(packet)
    } else {
      None
    }
  }

  /// Вспомогательный метод записи пакета
  pub async fn write_packet(&self, packet: ServersidePacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(packet).await
  }

  /// Вспомогательный метод записи `handshake` пакета
  pub async fn write_handshake_packet(&self, packet: ServersideHandshakePacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(ServersidePacket::Handshake(packet)).await
  }

  /// Вспомогательный метод записи `status` пакета
  pub async fn write_status_packet(&self, packet: ServersideStatusPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(ServersidePacket::Status(packet)).await
  }

  /// Вспомогательный метод записи `login` пакета
  pub async fn write_login_packet(&self, packet: ServersideLoginPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(ServersidePacket::Login(packet)).await
  }

  /// Вспомогательный метод записи `configuration` пакета
  pub async fn write_configuration_packet(&self, packet: ServersideConfigurationPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(ServersidePacket::Configuration(packet)).await
  }

  /// Вспомогательный метод записи `play` пакета
  pub async fn write_play_packet(&self, packet: ServersidePlayPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(ServersidePacket::Play(packet)).await
  }

  /// Метод выключения соединения
  pub async fn shutdown(&self) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.shutdown().await
  }

  /// Метод установки порога сжатия
  pub async fn set_compression_threshold(&self, threshold: i32) {
    self.compression_threshold.store(threshold, Ordering::SeqCst);
  }

  /// Устанавливает шифрование на соединении используя секретный ключ.
  /// Этот метод должен быть вызван **после** отправки `EncryptionResponse` серверу
  pub async fn set_encryption_key(&self, secret_key: [u8; 16]) {
    let (encryptor, decryptor) = nurtex_encrypt::create_cipher(&secret_key);

    {
      let reader = self.reader.lock().await;
      *reader.decryptor.lock().await = Some(decryptor);
    }

    {
      let writer = self.writer.lock().await;
      *writer.encryptor.lock().await = Some(encryptor);
    }
  }
}

impl From<TcpStream> for Connection {
  fn from(value: TcpStream) -> Self {
    let (read_stream, write_stream) = value.into_split();

    let state = Arc::new(AtomicI8::new(0));
    let compression_threshold = Arc::new(AtomicI32::new(-1));

    let reader = ConnectionReader {
      read_stream: Some(read_stream),
      buffer: BytesMut::with_capacity(64 * 1024),
      compression_threshold: Arc::clone(&compression_threshold),
      decryptor: Arc::new(Mutex::new(None)),
      state: Arc::clone(&state),
    };

    let writer = ConnectionWriter {
      write_stream: Some(write_stream),
      compression_threshold: Arc::clone(&compression_threshold),
      encryptor: Arc::new(Mutex::new(None)),
    };

    Self {
      reader: Arc::new(Mutex::new(reader)),
      writer: Arc::new(Mutex::new(writer)),
      state: state,
      compression_threshold,
    }
  }
}
