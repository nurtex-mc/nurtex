use nurtex_derive::PacketUnion;

use crate::packets::play::packets::*;

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ClientsidePlayPacket {
  #[id = 0x2B]
  KeepAlive(MultisideKeepAlive),
  #[id = 0x3B]
  Ping(ClientsidePing),
  #[id = 0x3C]
  PingResponse(ClientsidePingResponse),
  #[id = 0x19]
  DamageEvent(ClientsideDamageEvent),
  #[id = 0x33]
  UpdateEntityPos(ClientsideUpdateEntityPos),
  #[id = 0x36]
  UpdateEntityRot(ClientsideUpdateEntityRot),
  #[id = 0x34]
  UpdateEntityPosRot(ClientsideUpdateEntityPosRot),
  #[id = 0x46]
  PlayerPosition(ClientsidePlayerPosition),
  #[id = 0x47]
  PlayerRotation(ClientsidePlayerRotation),
  #[id = 0x45]
  PlayerLookAt(ClientsidePlayerLookAt),
  #[id = 0x42]
  PlayerCombatKill(ClientsidePlayerCombatKill),
  #[id = 0x66]
  SetHealth(ClientsideSetHealth),
  #[id = 0x65]
  SetExperience(ClientsideSetExperience),
  #[id = 0x69]
  SetPassengers(ClientsideSetPassengers),
  #[id = 0x63]
  SetEntityVelocity(ClientsideSetEntityVelocity),
  #[id = 0x01]
  SpawnEntity(ClientsideSpawnEntity),
  #[id = 0x4B]
  RemoveEntities(ClientsideRemoveEntities),
  #[id = 0x20]
  Disconnect(ClientsideDisconnect),
  #[id = 0x3F]
  PlayerChat(ClientsidePlayerChat),
  #[id = 0x77]
  SystemChat(ClientsideSystemChat),
  #[id = 0x7F]
  Transfer(ClientsideTransfer),
  #[id = 0x62]
  SetEntityLink(ClientsideSetEntityLink),
  #[id = 0x5D]
  ChunkCacheRadius(ClientsideChunkCacheRadius),
  #[id = 0x5C]
  ChunkCacheCenter(ClientsideChunkCacheCenter),
  #[id = 0x5B]
  SetCamera(ClientsideSetCamera),
  #[id = 0x51]
  RotateHead(ClientsideRotateHead),
  #[id = 0x52]
  SectionBlocksUpdate(ClientsideSectionBlocksUpdate),
  #[id = 0x4F]
  AddResourcePack(ClientsideAddResourcePack),
  #[id = 0x4E]
  RemoveResourcePack(ClientsideRemoveResourcePack),
  #[id = 0x4C]
  RemoveEntityEffect(ClientsideRemoveEntityEffect),
  #[id = 0x39]
  OpenContainer(ClientsideOpenContainer),
  #[id = 0x37]
  MoveVehicle(ClientsideMoveVehicle),
  #[id = 0x30]
  Login(ClientsideLogin),
  #[id = 0x23]
  EntityPositionSync(ClientsideEntityPositionSync),
  #[id = 0x24]
  Explosion(ClientsideExplosion),
  #[id = 0x25]
  UnloadChunk(ClientsideUnloadChunk),
  #[id = 0x26]
  GameEvent(ClientsideGameEvent),
  #[id = 0x1F]
  ClearChat(ClientsideClearChat),
  #[id = 0x0C]
  ChunkBatchStart(ClientsideChunkBatchStart),
  #[id = 0x0B]
  ChunkBatchFinished(ClientsideChunkBatchFinished),
  #[id = 0x2C]
  LoadChunkWithLight(ClientsideLoadChunkWithLight),
  #[id = 0x08]
  BlockUpdate(ClientsideBlockUpdate),
  #[id = 0x12]
  ContainerSetContent(ClientsideContainerSetContent),
  #[id = 0x14]
  ContainerSetSlot(ClientsideContainerSetSlot),
  #[id = 0x2F]
  LightUpdate(ClientsideLightUpdate),
  #[id = 0x11]
  CloseContainer(ClientsideCloseContainer),
  #[id = 0x13]
  ContainerSetData(ClientsideContainerSetData),
}

#[derive(Clone, Debug, PartialEq, PacketUnion)]
pub enum ServersidePlayPacket {
  #[id = 0x1B]
  KeepAlive(MultisideKeepAlive),
  #[id = 0x2C]
  Pong(ServersidePong),
  #[id = 0x25]
  PingRequest(ServersidePingRequest),
  #[id = 0x00]
  AcceptTeleportation(ServersideAcceptTeleportation),
  #[id = 0x3C]
  SwingArm(ServersideSwingArm),
  #[id = 0x40]
  UseItem(ServersideUseItem),
  #[id = 0x1D]
  MovePlayerPos(ServersideMovePlayerPos),
  #[id = 0x1F]
  MovePlayerRot(ServersideMovePlayerRot),
  #[id = 0x1E]
  MovePlayerPosRot(ServersideMovePlayerPosRot),
  #[id = 0x20]
  MovePlayerStatusOnly(ServersideMovePlayerStatusOnly),
  #[id = 0x0B]
  ClientCommand(ServersideClientCommand),
  #[id = 0x06]
  ChatCommand(ServersideChatCommand),
  #[id = 0x08]
  ChatMessage(ServersideChatMessage),
  #[id = 0x34]
  SetHeldItem(ServersideSetHeldItem),
  #[id = 0x19]
  Interact(ServersideInteract),
  #[id = 0x28]
  PlayerAction(ServersidePlayerAction),
  #[id = 0x29]
  PlayerCommand(ServersidePlayerCommand),
  #[id = 0x30]
  ResourcePackResponse(ServersideResourcePackResponse),
  #[id = 0x11]
  ContainerClick(ServersideContainerClick),
  #[id = 0x12]
  ContainerClose(ServersideContainerClose),
  #[id = 0x17]
  EditBook(ServersideEditBook),
}
