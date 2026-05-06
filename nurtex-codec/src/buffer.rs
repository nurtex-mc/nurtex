/// Трейт буффера
pub trait Buffer
where
  Self: Sized,
{
  /// Метод чтения типа из буффера
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self>;

  /// Метод записи типа в буффер
  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()>;
}
