use std::fmt::Debug;
use std::io::{Cursor, Error, ErrorKind};
use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicI32, Ordering};

use nurtex_encrypt::{AesDecryptor, AesEncryptor};
use nurtex_proxy::Proxy;
use nurtex_proxy::result::ProxyResult;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::Mutex;

use crate::connection::reader::{deserialize_packet, read_raw_packet};
use crate::connection::writer::{serialize_packet, write_raw_packet};
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

/// Структура для чтения пакетов
pub struct ConnectionReader {
  /// Специальная половина `TcpStream` для чтения пакетов
  read_stream: OwnedReadHalf,

  /// Текущий буффер данных
  buffer: Cursor<Vec<u8>>,

  /// Декодировщик данных
  decryptor: Arc<Mutex<Option<AesDecryptor>>>,

  /// Состояние подключения
  state: Arc<AtomicI8>,

  /// Порог сжатия (от 0 до 1024), изначально -1
  compression_threshold: Arc<AtomicI32>,
}

/// Структура для записи пакетов
pub struct ConnectionWriter {
  /// Специальная половина `TcpStream` для записи пакетов
  write_stream: OwnedWriteHalf,

  /// Кодировщик данных
  encryptor: Arc<Mutex<Option<AesEncryptor>>>,

  /// Порог сжатия (от 0 до 1024), изначально -1
  compression_threshold: Arc<AtomicI32>,
}

/// Основная структура подключения
pub struct NurtexConnection {
  /// Чтение пакетов
  reader: Arc<Mutex<ConnectionReader>>,

  /// Запись пакетов
  writer: Arc<Mutex<ConnectionWriter>>,

  /// Состояние подключения
  state: Arc<AtomicI8>,

  /// Порог сжатия (от 0 до 1024), изначально -1
  compression_threshold: Arc<AtomicI32>,
}

impl ConnectionReader {
  /// Метод чтения пакета
  pub async fn read_packet(&mut self) -> Option<ClientsidePacket> {
    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;

    let mut cursor = Cursor::new(raw_packet.as_ref());
    let state = ConnectionState::from(self.state.load(Ordering::SeqCst));

    match state {
      ConnectionState::Handshake => deserialize_packet::<ClientsideHandshakePacket>(&mut cursor).map(ClientsidePacket::Handshake),
      ConnectionState::Status => deserialize_packet::<ClientsideStatusPacket>(&mut cursor).map(ClientsidePacket::Status),
      ConnectionState::Login => deserialize_packet::<ClientsideLoginPacket>(&mut cursor).map(ClientsidePacket::Login),
      ConnectionState::Configuration => deserialize_packet::<ClientsideConfigurationPacket>(&mut cursor).map(ClientsidePacket::Configuration),
      ConnectionState::Play => deserialize_packet::<ClientsidePlayPacket>(&mut cursor).map(ClientsidePacket::Play),
    }
  }

  /// Вспомогательный метод чтения `status` пакета
  pub async fn read_status_packet(&mut self) -> Option<ClientsideStatusPacket> {
    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideStatusPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `login` пакета
  pub async fn read_login_packet(&mut self) -> Option<ClientsideLoginPacket> {
    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideLoginPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `configuration` пакета
  pub async fn read_configuration_packet(&mut self) -> Option<ClientsideConfigurationPacket> {
    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsideConfigurationPacket>(&mut cursor)
  }

  /// Вспомогательный метод чтения `play` пакета
  pub async fn read_play_packet(&mut self) -> Option<ClientsidePlayPacket> {
    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut decryptor_guard = self.decryptor.lock().await;

    let raw_packet = read_raw_packet(&mut self.read_stream, &mut self.buffer, compression_threshold, &mut *decryptor_guard).await?;
    let mut cursor = Cursor::new(raw_packet.as_ref());
    deserialize_packet::<ClientsidePlayPacket>(&mut cursor)
  }
}

impl ConnectionWriter {
  /// Метод записи пакета
  pub async fn write_packet(&mut self, packet: ServersidePacket) -> std::io::Result<()> {
    let serialized = match packet {
      ServersidePacket::Handshake(p) => serialize_packet(&p),
      ServersidePacket::Status(p) => serialize_packet(&p),
      ServersidePacket::Login(p) => serialize_packet(&p),
      ServersidePacket::Configuration(p) => serialize_packet(&p),
      ServersidePacket::Play(p) => serialize_packet(&p),
    }
    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to serialize packet"))?;

    let compression_threshold = self.compression_threshold.load(Ordering::SeqCst);
    let mut encryptor_guard = self.encryptor.lock().await;

    write_raw_packet(&serialized, &mut self.write_stream, compression_threshold, &mut *encryptor_guard).await
  }

  /// Вспомогательный метод записи `handshake` пакета
  pub async fn write_handshake_packet(&mut self, packet: ServersideHandshakePacket) -> std::io::Result<()> {
    self.write_packet(ServersidePacket::Handshake(packet)).await
  }

  /// Вспомогательный метод записи `status` пакета
  pub async fn write_status_packet(&mut self, packet: ServersideStatusPacket) -> std::io::Result<()> {
    self.write_packet(ServersidePacket::Status(packet)).await
  }

  /// Вспомогательный метод записи `login` пакета
  pub async fn write_login_packet(&mut self, packet: ServersideLoginPacket) -> std::io::Result<()> {
    self.write_packet(ServersidePacket::Login(packet)).await
  }

  /// Вспомогательный метод записи `configuration` пакета
  pub async fn write_configuration_packet(&mut self, packet: ServersideConfigurationPacket) -> std::io::Result<()> {
    self.write_packet(ServersidePacket::Configuration(packet)).await
  }

  /// Вспомогательный метод записи `play` пакета
  pub async fn write_play_packet(&mut self, packet: ServersidePlayPacket) -> std::io::Result<()> {
    self.write_packet(ServersidePacket::Play(packet)).await
  }

  /// Метод выключения потока записи
  pub async fn shutdown(&mut self) -> std::io::Result<()> {
    self.write_stream.shutdown().await
  }
}

impl NurtexConnection {
  /// Метод создания нового подключения
  pub async fn new(server_host: impl Into<String>, server_port: u16) -> std::io::Result<Self> {
    let stream = TcpStream::connect(format!("{}:{}", server_host.into(), server_port)).await?;
    stream.set_nodelay(true)?;
    Self::new_from_stream(stream).await
  }

  /// Метод создания нового подключения с прокси
  pub async fn new_with_proxy(server_host: impl Into<String>, server_port: u16, proxy: &Proxy) -> std::io::Result<Self> {
    proxy.bind(server_host.into(), server_port);

    let stream = match proxy.connect().await {
      ProxyResult::Ok(s) => s,
      ProxyResult::Err(e) => return Err(Error::new(ErrorKind::NotConnected, e.text())),
    };

    stream.set_nodelay(true)?;

    Self::new_from_stream(stream).await
  }

  /// Метод создания нового подключения из TcpStream
  pub async fn new_from_stream(stream: TcpStream) -> std::io::Result<Self> {
    let (rh, wh) = stream.into_split();

    // Изначально хэндшейк
    let state = Arc::new(AtomicI8::new(0));

    // Изначально отрицательный (не указан)
    let compression_threshold = Arc::new(AtomicI32::new(-1));

    let reader = ConnectionReader {
      read_stream: rh,
      buffer: Cursor::new(Vec::new()),
      compression_threshold: Arc::clone(&compression_threshold),
      decryptor: Arc::new(Mutex::new(None)),
      state: Arc::clone(&state),
    };

    let writer = ConnectionWriter {
      write_stream: wh,
      compression_threshold: Arc::clone(&compression_threshold),
      encryptor: Arc::new(Mutex::new(None)),
    };

    Ok(NurtexConnection {
      reader: Arc::new(Mutex::new(reader)),
      writer: Arc::new(Mutex::new(writer)),
      state: state,
      compression_threshold,
    })
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
    reader.read_status_packet().await
  }

  /// Вспомогательный метод чтения `login` пакета
  pub async fn read_login_packet(&self) -> Option<ClientsideLoginPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_login_packet().await
  }

  /// Вспомогательный метод чтения `configuration` пакета
  pub async fn read_configuration_packet(&self) -> Option<ClientsideConfigurationPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_configuration_packet().await
  }

  /// Вспомогательный метод чтения `play` пакета
  pub async fn read_play_packet(&self) -> Option<ClientsidePlayPacket> {
    let mut reader = self.reader.lock().await;
    reader.read_play_packet().await
  }

  /// Вспомогательный метод записи пакета
  pub async fn write_packet(&self, packet: ServersidePacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_packet(packet).await
  }

  /// Вспомогательный метод записи `handshake` пакета
  pub async fn write_handshake_packet(&self, packet: ServersideHandshakePacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_handshake_packet(packet).await
  }

  /// Вспомогательный метод записи `status` пакета
  pub async fn write_status_packet(&self, packet: ServersideStatusPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_status_packet(packet).await
  }

  /// Вспомогательный метод записи `login` пакета
  pub async fn write_login_packet(&self, packet: ServersideLoginPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_login_packet(packet).await
  }

  /// Вспомогательный метод записи `configuration` пакета
  pub async fn write_configuration_packet(&self, packet: ServersideConfigurationPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_configuration_packet(packet).await
  }

  /// Вспомогательный метод записи `play` пакета
  pub async fn write_play_packet(&self, packet: ServersidePlayPacket) -> std::io::Result<()> {
    let mut writer = self.writer.lock().await;
    writer.write_play_packet(packet).await
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
