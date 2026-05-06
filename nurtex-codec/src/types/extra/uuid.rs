use uuid::Uuid;

use crate::Buffer;

impl Buffer for Uuid {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let array = [u32::read_buf(buffer)?, u32::read_buf(buffer)?, u32::read_buf(buffer)?, u32::read_buf(buffer)?];

    let most = ((array[0] as u64) << 32) | ((array[1] as u64) & 0xffffffff);
    let least = ((array[2] as u64) << 32) | ((array[3] as u64) & 0xffffffff);

    Some(Uuid::from_u128(((most as u128) << 64) | least as u128))
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let most = (self.as_u128() >> 64) as u64;
    let least = (self.as_u128() & 0xffffffffffffffff) as u64;

    let [a, b, c, d] = [(most >> 32) as u32, most as u32, (least >> 32) as u32, least as u32];

    a.write_buf(buffer)?;
    b.write_buf(buffer)?;
    c.write_buf(buffer)?;
    d.write_buf(buffer)?;
    Ok(())
  }
}
