use nurtex_derive::PacketUnion;

use crate::packets::status::packets::*;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideStatusPacket {
  #[id = 0x00]
  StatusResponse(ClientsideStatusResponse),
  #[id = 0x01]
  PongResponse(ClientsidePongResponse),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideStatusPacket {
  #[id = 0x00]
  StatusRequest(ServersideStatusRequest),
  #[id = 0x01]
  PingRequest(ServersidePingRequest),
}
