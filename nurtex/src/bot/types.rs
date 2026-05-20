use std::sync::Arc;

use tokio::sync::broadcast;

/// Тип потокобезопасного `reader`
pub type PacketReader = Arc<broadcast::Sender<crate::protocol::connection::ClientsidePacket>>;

/// Тип потокобезопасного `writer`
pub type PacketWriter = Arc<broadcast::Sender<crate::protocol::packets::play::ServersidePlayPacket>>;
