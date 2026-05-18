use std::fmt::Debug;
use std::io::Read;

use bytes::{Bytes, BytesMut};
use flate2::Compression;
use flate2::bufread::ZlibEncoder;
use nurtex_codec::types::variable::VarI32;
use nurtex_encrypt::AesEncryptor;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::ProtocolPacket;

/// Функция записи сетевого пакета
pub async fn write_packet<P, W>(packet: &P, stream: &mut W, compression_threshold: i32, cipher: &mut Option<AesEncryptor>) -> std::io::Result<()>
where
  P: ProtocolPacket + Debug,
  W: AsyncWrite + Unpin + Send,
{
  let raw_packet = serialize_packet(packet)?;
  write_raw_packet(&raw_packet, stream, compression_threshold, cipher).await
}

/// Функция сериализации пакета
pub fn serialize_packet<P>(packet: &P) -> std::io::Result<Bytes>
where
  P: ProtocolPacket + Debug,
{
  let mut buf = Vec::new();

  (packet.id() as i32)
    .write_var(&mut buf)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

  packet.write(&mut buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

  if buf.len() > 8388608 {
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "packet too large"));
  }

  Ok(Bytes::from(buf))
}

/// Функция записи сырого пакета
pub async fn write_raw_packet<W>(raw_packet: &[u8], stream: &mut W, compression_threshold: i32, cipher: &mut Option<AesEncryptor>) -> std::io::Result<()>
where
  W: AsyncWrite + Unpin + Send,
{
  let network_packet = encode_to_network_packet(raw_packet, compression_threshold, cipher)?;
  stream.write_all(&network_packet).await
}

/// Функция кодировки байтов в сетевой пакет
pub fn encode_to_network_packet(raw_packet: &[u8], compression_threshold: i32, cipher: &mut Option<AesEncryptor>) -> std::io::Result<BytesMut> {
  let mut buf = BytesMut::new();

  if compression_threshold >= 0 {
    compression_encoder(raw_packet, compression_threshold, &mut buf)?;
  } else {
    buf.extend_from_slice(raw_packet);
  }

  let mut frame = BytesMut::new();
  let mut length = Vec::new();

  (buf.len() as i32)
    .write_var(&mut length)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

  frame.extend_from_slice(&length);
  frame.extend_from_slice(&buf);

  if let Some(cipher) = cipher {
    nurtex_encrypt::encrypt_data(cipher, &mut frame);
  }

  Ok(frame)
}

/// Функция кодировки с учётом порога сжатия
pub fn compression_encoder(data: &[u8], compression_threshold: i32, buf: &mut BytesMut) -> std::io::Result<()> {
  let n = data.len();

  if n < compression_threshold as usize {
    let mut temp = Vec::new();
    0.write_var(&mut temp).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    buf.extend_from_slice(&temp);
    buf.extend_from_slice(data);
  } else {
    let mut deflater = ZlibEncoder::new(data, Compression::default());
    let mut compressed_data = Vec::new();
    deflater.read_to_end(&mut compressed_data)?;

    let mut length = Vec::new();

    (n as i32).write_var(&mut length).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    buf.extend_from_slice(&length);
    buf.extend_from_slice(&compressed_data);
  }

  Ok(())
}
