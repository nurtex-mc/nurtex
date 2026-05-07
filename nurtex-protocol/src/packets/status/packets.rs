use nurtex_codec::Buffer;
use nurtex_derive::Packet;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsidePongResponse {
  pub timestamp: i64,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ClientsideStatusResponse {
  pub response: String,
}

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersideStatusRequest;

#[derive(Clone, Debug, PartialEq, Packet)]
pub struct ServersidePingRequest {
  pub timestamp: i64,
}