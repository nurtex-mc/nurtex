use std::fmt::Debug;
use std::io::Read;

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
  let raw_packet = serialize_packet(packet).unwrap();
  write_raw_packet(&raw_packet, stream, compression_threshold, cipher).await
}

/// Функция сериализации пакета
pub fn serialize_packet<P>(packet: &P) -> Option<Box<[u8]>>
where
  P: ProtocolPacket + Debug,
{
  let mut buf = Vec::new();
  (packet.id() as i32).write_var(&mut buf).ok()?;
  packet.write(&mut buf).ok()?;

  if buf.len() > 8388608 as usize {
    return None;
  }

  Some(buf.into_boxed_slice())
}

/// Функция записи сырого пакета
pub async fn write_raw_packet<W>(raw_packet: &[u8], stream: &mut W, compression_threshold: i32, cipher: &mut Option<AesEncryptor>) -> std::io::Result<()>
where
  W: AsyncWrite + Unpin + Send,
{
  let network_packet = encode_to_network_packet(raw_packet, compression_threshold, cipher);
  stream.write_all(&network_packet).await
}

/// Функция кодировки байтов в сетевой пакет
pub fn encode_to_network_packet(raw_packet: &[u8], compression_threshold: i32, cipher: &mut Option<AesEncryptor>) -> Vec<u8> {
  let mut raw_packet = raw_packet.to_vec();

  if compression_threshold >= 0 {
    raw_packet = compression_encoder(&raw_packet, compression_threshold).unwrap();
  }

  raw_packet = frame_prepender(raw_packet).unwrap();

  if let Some(cipher) = cipher {
    nurtex_encrypt::encrypt_packet(cipher, &mut raw_packet);
  }

  raw_packet
}

/// Функция кодировки с учётом порога сжатия
pub fn compression_encoder(data: &[u8], compression_threshold: i32) -> Option<Vec<u8>> {
  let n = data.len();

  if n < compression_threshold as usize {
    let mut buf = Vec::new();

    0i32.write_var(&mut buf).ok()?;
    std::io::Write::write_all(&mut buf, data).ok()?;

    Some(buf)
  } else {
    let mut deflater = ZlibEncoder::new(data, Compression::default());
    let mut compressed_data = Vec::new();
    deflater.read_to_end(&mut compressed_data).ok()?;

    let mut len_prepended_compressed_data = Vec::new();
    (data.len() as i32).write_var(&mut len_prepended_compressed_data).ok()?;
    len_prepended_compressed_data.append(&mut compressed_data);

    Some(len_prepended_compressed_data)
  }
}

/// Функция подготовки фрейма
fn frame_prepender(mut data: Vec<u8>) -> Option<Vec<u8>> {
  let mut buf = Vec::new();

  (data.len() as i32).write_var(&mut buf).ok()?;
  buf.append(&mut data);

  Some(buf)
}
