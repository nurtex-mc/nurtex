use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{RwLock, broadcast};

use crate::BotProfile;
use crate::bot::BotComponents;
use crate::bot::capture::{capture_components, capture_connection};
use crate::bot::handlers::{ChatPayload, DisconnectPayload, Handlers};
use crate::bot::plugins::Plugins;
use crate::bot::types::{PacketReader, PacketWriter};
use crate::protocol::connection::Connection;
use crate::protocol::connection::utils::handle_encryption_request;
use crate::protocol::connection::{ClientsidePacket, ConnectionState};
use crate::protocol::packets::play::{ClientsidePlayPacket, ServersideAcceptTeleportation, ServersideClientCommand};
use crate::protocol::packets::{
  configuration::{ClientsideConfigurationPacket, ServersideAcknowledgeFinishConfiguration, ServersideConfigurationPacket, ServersideKnownPacks},
  handshake::{ServersideGreet, ServersideHandshakePacket},
  login::{ClientsideLoginPacket, ServersideLoginAcknowledged, ServersideLoginPacket, ServersideLoginStart},
  play::ServersidePlayPacket,
};
use crate::protocol::types::Chunk;
use crate::protocol::types::{ClientCommand, ClientIntention, ResourcePackState, Rotation, Vector3};
use crate::registry::EntityKind;
use crate::storage::Storage;
use crate::world::{Entity, EntityId};

#[cfg(feature = "proxy")]
use crate::proxy::Proxy;

#[cfg(feature = "speedometer")]
use crate::speedometer::Speedometer;

/// Структура основных данных бота
pub struct BotPackage {
  /// TCP-подключение
  pub connection: Arc<RwLock<Connection>>,

  /// Юзернейм бота
  pub profile: Arc<RwLock<BotProfile>>,

  /// Компоненты бота
  pub components: Arc<RwLock<BotComponents>>,

  /// Идентификатор сущности бота
  pub entity_id: Arc<EntityId>,

  /// Спидометр
  #[cfg(feature = "speedometer")]
  pub speedometer: Option<Arc<Speedometer>>,

  /// Плагины бота
  pub plugins: Arc<Plugins>,

  /// Читатель пакетов
  pub packet_reader: PacketReader,

  /// Записыватель пакетов
  pub packet_writer: PacketWriter,

  /// Хранилище бота
  pub storage: Arc<Storage>,

  /// Прокси бота
  #[cfg(feature = "proxy")]
  pub proxy: Option<Arc<Proxy>>,

  /// Обработчики событий бота
  pub handlers: Arc<Handlers>,

  /// Хост целевого сервера
  pub server_host: String,

  /// Порт целевого сервера
  pub server_port: u16,

  /// Версия протокола
  pub protocol_version: i32,

  /// Таймаут подключения к серверу
  pub connection_timeout: u64,
}

/// Функция спавна процесса подключения
pub async fn spawn_connection(package: &BotPackage) -> std::io::Result<()> {
  let connection = &package.connection;

  {
    let conn_guard = connection.read().await;
    conn_guard.shutdown().await?;
  }

  #[cfg(feature = "proxy")]
  match &package.proxy {
    Some(proxy) => match tokio::time::timeout(
      Duration::from_millis(package.connection_timeout),
      connection.read().await.connect_with_proxy(package.server_host.clone(), package.server_port, proxy),
    )
    .await
    {
      Ok(result) => match result {
        Ok(c) => c,
        Err(err) => return Err(err),
      },
      Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed to receive a response from server within specified timeout")),
    },
    None => match tokio::time::timeout(
      Duration::from_millis(package.connection_timeout),
      connection.read().await.connect(package.server_host.clone(), package.server_port),
    )
    .await
    {
      Ok(result) => match result {
        Ok(c) => c,
        Err(err) => return Err(err),
      },
      Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed to receive a response from server within specified timeout")),
    },
  }

  #[cfg(not(feature = "proxy"))]
  match tokio::time::timeout(
    Duration::from_millis(package.connection_tiemout),
    connection.read().await.connect(package.server_host.clone(), server_port),
  )
  .await
  {
    Ok(result) => match result {
      Ok(c) => c,
      Err(err) => return Err(err),
    },
    Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed to receive a response from server within specified timeout")),
  }

  let profile_data = { package.profile.read().await.clone() };

  capture_connection(&connection, async |conn| {
    conn
      .write_handshake_packet(ServersideHandshakePacket::Greet(ServersideGreet {
        protocol_version: package.protocol_version,
        server_host: package.server_host.clone(),
        server_port: package.server_port,
        intention: ClientIntention::Login,
      }))
      .await?;

    conn.set_state(ConnectionState::Login).await;

    conn
      .write_login_packet(ServersideLoginPacket::LoginStart(ServersideLoginStart {
        username: profile_data.username.clone(),
        uuid: profile_data.uuid,
      }))
      .await
  })
  .await?;

  loop {
    let Some(packet) = ({
      let conn_guard = connection.read().await;
      conn_guard.read_login_packet().await
    }) else {
      continue;
    };

    match packet {
      ClientsideLoginPacket::Compression(p) => {
        capture_connection(&connection, async |conn| {
          conn.set_compression_threshold(p.compression_threshold).await;
          Ok(())
        })
        .await?;
      }
      ClientsideLoginPacket::EncryptionRequest(request) => {
        if let Some((response, secret_key)) = handle_encryption_request(&request) {
          capture_connection(&connection, async |conn| {
            conn.write_login_packet(ServersideLoginPacket::EncryptionResponse(response)).await?;
            conn.set_encryption_key(secret_key).await;
            Ok(())
          })
          .await?;
        }
      }
      ClientsideLoginPacket::LoginSuccess(p) => {
        if let Some(handler) = &package.handlers.on_login_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(username_clone));
        }

        package.profile.write().await.uuid = p.uuid;

        capture_connection(&connection, async |conn| {
          conn.write_login_packet(ServersideLoginPacket::LoginAcknowledged(ServersideLoginAcknowledged)).await
        })
        .await?;

        break;
      }
      ClientsideLoginPacket::Disconnect(_p) => {
        if let Some(handler) = &package.handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(username_clone, DisconnectPayload { state: ConnectionState::Play }));
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
      }
      _ => {}
    }
  }

  capture_connection(&connection, async |conn| {
    conn.set_state(ConnectionState::Configuration).await;
    conn
      .write_configuration_packet(ServersideConfigurationPacket::ClientInformation(
        package.profile.read().await.information.to_serverside_packet(),
      ))
      .await
  })
  .await?;

  loop {
    let Some(packet) = ({
      let conn_guard = connection.read().await;
      conn_guard.read_configuration_packet().await
    }) else {
      continue;
    };

    match packet {
      ClientsideConfigurationPacket::KeepAlive(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::KeepAlive(crate::protocol::packets::configuration::MultisideKeepAlive {
              id: p.id,
            }))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::Ping(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::Pong(crate::protocol::packets::configuration::ServersidePong { id: p.id }))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::KnownPacks(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::KnownPacks(ServersideKnownPacks { known_packs: p.known_packs }))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::FinishConfiguration(_) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::AcknowledgeFinishConfiguration(ServersideAcknowledgeFinishConfiguration))
            .await
        })
        .await?;

        break;
      }
      ClientsideConfigurationPacket::AddResourcePack(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_configuration_packet(ServersideConfigurationPacket::ResourcePackResponse(
              crate::protocol::packets::configuration::ServersideResourcePackResponse {
                uuid: p.uuid,
                state: ResourcePackState::Accepted,
              },
            ))
            .await
        })
        .await?;
      }
      ClientsideConfigurationPacket::Disconnect(_p) => {
        if let Some(handler) = &package.handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(username_clone, DisconnectPayload { state: ConnectionState::Play }));
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
      }
      _ => {}
    }
  }

  capture_connection(&connection, async |conn| {
    conn.set_state(ConnectionState::Play).await;
    Ok(())
  })
  .await?;

  #[cfg(feature = "speedometer")]
  if let Some(speedometer) = &package.speedometer {
    speedometer.bot_joined(profile_data.username.clone());
  }

  if let Some(handler) = &package.handlers.on_spawn_handler {
    let username_clone = profile_data.username.clone();
    let handler_clone = Arc::clone(handler);

    tokio::spawn(handler_clone(username_clone));
  }

  let mut packet_rx = {
    let reader_tx = Arc::clone(&package.packet_reader);
    reader_tx.subscribe()
  };

  loop {
    let packet = match tokio::time::timeout(Duration::from_millis(8000), packet_rx.recv()).await {
      Ok(Ok(ClientsidePacket::Play(play_packet))) => play_packet,
      Ok(Ok(_)) => continue,
      Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
      Ok(Err(broadcast::error::RecvError::Closed)) => return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server")),
      Err(_) => continue,
    };

    match packet {
      ClientsidePlayPacket::Login(p) => {
        package.entity_id.set(p.entity_id);

        if package.plugins.auto_respawn.enabled && p.enable_respawn_screen {
          tokio::time::sleep(Duration::from_millis(package.plugins.auto_respawn.respawn_delay)).await;

          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::ClientCommand(ServersideClientCommand {
                command: ClientCommand::PerformRespawn,
              }))
              .await
          })
          .await?;
        }
      }
      ClientsidePlayPacket::SpawnEntity(p) => {
        let entity = Entity {
          kind: if let Some(k) = EntityKind::from_id(p.entity_type) { k } else { EntityKind::Null },
          uuid: p.entity_uuid,
          position: p.position,
          rotation: Rotation::from_angle(p.yaw_angle, p.pitch_angle),
          velocity: p.velocity.to_vector3(),
          ..Default::default()
        };

        package.storage.add_entity(p.entity_id, entity).await;
      }
      ClientsidePlayPacket::LoadChunkWithLight(p) => {
        if let Some(chunk) = Chunk::decode_to_end(p.chunk_x, p.chunk_z, &p.chunk_data, -64) {
          package.storage.add_chunk(chunk).await;
        }
      }
      ClientsidePlayPacket::RemoveEntities(p) => {
        package
          .storage
          .capture_entities(async |entities| {
            p.entities.iter().for_each(|entity_id| {
              entities.remove(entity_id);
            });
          })
          .await;
      }
      ClientsidePlayPacket::EntityPositionSync(p) => {
        package
          .storage
          .capture_entity(&p.entity_id, async |entity| {
            entity.position = p.position;
            entity.rotation = p.rotation;
            entity.velocity = p.velocity;
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityPos(p) => {
        package
          .storage
          .capture_entity(&p.entity_id, async |entity| {
            // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityRot(p) => {
        package
          .storage
          .capture_entity(&p.entity_id, async |entity| {
            entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::UpdateEntityPosRot(p) => {
        package
          .storage
          .capture_entity(&p.entity_id, async |entity| {
            // entity.position.with_delta(p.delta_x, p.delta_y, p.delta_z);
            entity.rotation = Rotation::from_angle(p.yaw_angle, p.pitch_angle);
            entity.on_ground = p.on_ground;
          })
          .await;
      }
      ClientsidePlayPacket::SetEntityVelocity(p) => {
        if package.entity_id.get() == p.entity_id {
          capture_components(&package.components, async |comp| {
            comp.velocity = p.velocity.to_vector3();
          })
          .await;
        } else {
          package
            .storage
            .capture_entity(&p.entity_id, async |entity| {
              entity.position.with_velocity(p.velocity.to_vector3());
              entity.velocity = Vector3::from_lp_vector3(p.velocity);
            })
            .await;
        }
      }
      ClientsidePlayPacket::KeepAlive(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::KeepAlive(crate::protocol::packets::play::MultisideKeepAlive { id: p.id }))
            .await
        })
        .await?;
      }
      ClientsidePlayPacket::PlayerChat(p) => {
        if let Some(handler) = &package.handlers.on_chat_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(
            username_clone,
            ChatPayload {
              message: p.message,
              sender_uuid: p.sender_uuid,
            },
          ));
        }
      }
      ClientsidePlayPacket::Ping(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::Pong(crate::protocol::packets::play::ServersidePong { id: p.id }))
            .await
        })
        .await?;
      }
      ClientsidePlayPacket::SetHealth(p) => {
        capture_components(&package.components, async |comp| {
          comp.health = p.health;
          comp.food = p.food;
        })
        .await;
      }
      ClientsidePlayPacket::SetExperience(p) => {
        capture_components(&package.components, async |comp| {
          comp.experience = p.experience;
        })
        .await;
      }
      ClientsidePlayPacket::PlayerPosition(p) => {
        capture_components(&package.components, async |comp| {
          comp.position = p.position;
          comp.velocity = p.velocity;
          comp.rotation = p.rotation;
        })
        .await;

        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::AcceptTeleportation(ServersideAcceptTeleportation { teleport_id: p.teleport_id }))
            .await
        })
        .await?;
      }
      ClientsidePlayPacket::PlayerRotation(p) => {
        capture_components(&package.components, async |comp| {
          comp.rotation = Rotation::new(p.yaw, p.pitch);
        })
        .await;
      }
      ClientsidePlayPacket::AddResourcePack(p) => {
        capture_connection(&connection, async |conn| {
          conn
            .write_play_packet(ServersidePlayPacket::ResourcePackResponse(crate::protocol::packets::play::ServersideResourcePackResponse {
              uuid: p.uuid,
              state: ResourcePackState::Accepted,
            }))
            .await
        })
        .await?;
      }
      ClientsidePlayPacket::PlayerCombatKill(_p) => {
        if let Some(handler) = &package.handlers.on_death_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(username_clone));
        }

        if package.plugins.auto_respawn.enabled {
          tokio::time::sleep(Duration::from_millis(package.plugins.auto_respawn.respawn_delay)).await;

          capture_connection(&connection, async |conn| {
            conn
              .write_play_packet(ServersidePlayPacket::ClientCommand(ServersideClientCommand {
                command: ClientCommand::PerformRespawn,
              }))
              .await
          })
          .await?;
        }
      }
      ClientsidePlayPacket::Disconnect(_p) => {
        if let Some(handler) = &package.handlers.on_disconnect_handler {
          let username_clone = profile_data.username.clone();
          let handler_clone = Arc::clone(handler);

          tokio::spawn(handler_clone(username_clone, DisconnectPayload { state: ConnectionState::Play }));
        }

        return Err(Error::new(ErrorKind::ConnectionReset, "connection was reset by server"));
      }
      _ => {}
    }
  }
}
