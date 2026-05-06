use nurtex_derive::PacketUnion;

use crate::packets::configuration::packets::*;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsideConfigurationPacket {
  #[id = 0x00]
  CookieRequest(ClientsideCookieRequest),
  #[id = 0x01]
  PluginMessage(ClientsidePluginMessage),
  #[id = 0x02]
  Disconnect(ClientsideDisconnect),
  #[id = 0x03]
  FinishConfiguration(ClientsideFinishConfiguration),
  #[id = 0x04]
  KeepAlive(MultisideKeepAlive),
  #[id = 0x05]
  Ping(ClientsidePing),
  #[id = 0x06]
  ResetChat(ClientsideResetChat),
  #[id = 0x07]
  RegistryData(ClientsideRegistryData),
  #[id = 0x08]
  RemoveResourcePack(ClientsideRemoveResourcePack),
  #[id = 0x09]
  AddResourcePack(ClientsideAddResourcePack),
  #[id = 0x0A]
  StoreCookie(ClientsideStoreCookie),
  #[id = 0x0B]
  Transfer(ClientsideTransfer),
  #[id = 0x0C]
  FeatureFlags(ClientsideFeatureFlags),
  #[id = 0x0D]
  UpdateTags(ClientsideUpdateTags),
  #[id = 0x0E]
  KnownPacks(ClientsideKnownPacks),
  #[id = 0x0F]
  CustomReportDetails(ClientsideCustomReportDetails),
  #[id = 0x10]
  ServerLinks(ClientsideServerLinks),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersideConfigurationPacket {
  #[id = 0x00]
  ClientInformation(ServersideClientInformation),
  #[id = 0x01]
  CookieResponse(ServersideCookieResponse),
  #[id = 0x02]
  PluginMessage(ServersidePluginMessage),
  #[id = 0x03]
  AcknowledgeFinishConfiguration(ServersideAcknowledgeFinishConfiguration),
  #[id = 0x04]
  KeepAlive(MultisideKeepAlive),
  #[id = 0x05]
  Pong(ServersidePong),
  #[id = 0x06]
  ResourcePackResponse(ServersideResourcePackResponse),
  #[id = 0x07]
  KnownPacks(ServersideKnownPacks),
}
