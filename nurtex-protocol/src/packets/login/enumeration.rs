use nurtex_derive::PacketUnion;

use crate::packets::login::packets::*;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideLoginPacket {
  #[id = 0x00]
  Disconnect(ClientsideLoginDisconnect),
  #[id = 0x01]
  EncryptionRequest(ClientsideEncryptionRequest),
  #[id = 0x02]
  LoginSuccess(ClientsideLoginSuccess),
  #[id = 0x03]
  Compression(ClientsideSetCompression),
  #[id = 0x04]
  PluginRequest(ClientsidePluginRequest),
  #[id = 0x05]
  CookieRequest(ClientsideCookieRequest),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideLoginPacket {
  #[id = 0x00]
  LoginStart(ServersideLoginStart),
  #[id = 0x01]
  EncryptionResponse(ServersideEncryptionResponse),
  #[id = 0x02]
  PluginResponse(ServersidePluginResponse),
  #[id = 0x03]
  LoginAcknowledged(ServersideLoginAcknowledged),
  #[id = 0x04]
  CookieResponse(ServersideCookieResponse),
}
