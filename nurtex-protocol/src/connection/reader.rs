use std::fmt::Debug;
use std::io::{Cursor, Read};

use bytes::{Buf, Bytes, BytesMut};
use flate2::read::ZlibDecoder;
use futures::StreamExt;
use nurtex_codec::types::variable::VarI32;
use nurtex_encrypt::AesDecryptor;
use tokio::io::AsyncRead;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::ProtocolPacket;

/// Функция парсинга фрейма
fn parse_frame(buf: &mut BytesMut) -> Option<Bytes> {
  let mut temp_buf = Cursor::new(&buf[..]);

  let length = i32::read_var(&mut temp_buf)? as usize;
  let varint_length = temp_buf.position() as usize;

  if buf.len() < varint_length + length {
    return None;
  }

  buf.advance(varint_length);

  let data = buf.split_to(length).freeze();

  if buf.is_empty() && buf.capacity() > 64 * 1024 {
    buf.clear();
    buf.reserve(64 * 1024);
  }

  Some(data)
}

/// Функция десериализации сетевого пакета
pub fn deserialize_packet<P: ProtocolPacket + Debug>(buf: &mut Cursor<&[u8]>) -> Option<P> {
  let packet_id = i32::read_var(buf)? as u32;
  P::read(packet_id, buf)
}

/// Функция декодировки с учётом порога сжатия
pub fn compression_decoder(data: &[u8], compression_threshold: i32) -> Option<Bytes> {
  let mut cursor = Cursor::new(data);
  let n = i32::read_var(&mut cursor)?;

  if n == 0 {
    let remaining = &data[cursor.position() as usize..];
    return Some(Bytes::copy_from_slice(remaining));
  }

  if n < compression_threshold || n > 8388608 {
    return None;
  }

  let mut buf = Vec::with_capacity(n as usize);
  let mut decoder = ZlibDecoder::new(&data[cursor.position() as usize..]);
  decoder.read_to_end(&mut buf).ok()?;

  Some(Bytes::from(buf))
}

/// Функция чтения сетевого пакета
pub async fn read_packet<P: ProtocolPacket + Debug, R>(stream: &mut R, buf: &mut BytesMut, compression_threshold: i32, cipher: &mut Option<AesDecryptor>) -> Option<P>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  let raw_packet = read_raw_packet(stream, buf, compression_threshold, cipher).await?;
  let packet = deserialize_packet(&mut Cursor::new(&raw_packet[..]))?;
  Some(packet)
}

/// Функция чтения сырого пакета
pub async fn read_raw_packet<R>(stream: &mut R, buf: &mut BytesMut, compression_threshold: i32, cipher: &mut Option<AesDecryptor>) -> Option<Bytes>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  loop {
    if let Some(packet) = read_raw_packet_from_buffer(buf, compression_threshold) {
      return Some(packet);
    };

    let bytes = read_and_decrypt_frame(stream, cipher).await?;
    buf.extend_from_slice(&bytes);
  }
}

/// Функция чтения и расшифровки фрейма
async fn read_and_decrypt_frame<R>(stream: &mut R, cipher: &mut Option<AesDecryptor>) -> Option<Bytes>
where
  R: AsyncRead + Unpin + Send + Sync,
{
  let mut framed = FramedRead::new(stream, BytesCodec::new());

  let mut bytes = framed.next().await?.ok()?;

  if let Some(cipher) = cipher {
    nurtex_encrypt::decrypt_data(cipher, &mut bytes);
  }

  Some(bytes.freeze())
}

/// Функция чтения сырого пакета из буффера
pub fn read_raw_packet_from_buffer(buf: &mut BytesMut, compression_threshold: i32) -> Option<Bytes> {
  let frame = parse_frame(buf)?;

  if compression_threshold >= 0 {
    compression_decoder(&frame, compression_threshold)
  } else {
    Some(frame)
  }
}
