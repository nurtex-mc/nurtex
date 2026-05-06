use nurtex_derive::PacketUnion;

use crate::packets::handshake::packets::*;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideHandshakePacket {
  #[id = 0x00]
  Greet(ServersideGreet),
}

// В состоянии Handshake нету Clientside пакетов
#[derive(Clone, Debug, PartialEq)]
pub enum ClientsideHandshakePacket {}

impl crate::ProtocolPacket for ClientsideHandshakePacket {
  fn id(&self) -> u32 {
    match *self {}
  }

  fn read(_id: u32, _buf: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    None
  }

  fn write(&self, _buf: &mut impl std::io::Write) -> std::io::Result<()> {
    match *self {}
  }
}
