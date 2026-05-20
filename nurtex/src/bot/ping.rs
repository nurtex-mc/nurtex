use std::io::{Error, ErrorKind};
use std::time::Duration;

use tokio::time::timeout;

use crate::protocol::connection::Connection;
use crate::proxy::Proxy;

/// Функция пингования сервера
pub async fn ping_server(server_host: impl Into<String>, server_port: u16) -> std::io::Result<u128> {
  let start_time = std::time::Instant::now();

  let conn = Connection::new();

  match timeout(Duration::from_secs(30), conn.connect(server_host.into(), server_port)).await {
    Ok(r) => match r {
      Ok(_) => {}
      Err(e) => return Err(e),
    },
    Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed connect to server")),
  }

  let elapsed_time = start_time.elapsed();

  Ok(elapsed_time.as_millis())
}

/// Функция пингования сервера с прокси
pub async fn ping_server_with_proxy(server_host: impl Into<String>, server_port: u16, proxy: &Proxy) -> std::io::Result<u128> {
  let start_time = std::time::Instant::now();

  let conn = Connection::new();

  match timeout(Duration::from_secs(30), conn.connect_with_proxy(server_host.into(), server_port, proxy)).await {
    Ok(r) => match r {
      Ok(_) => {}
      Err(e) => return Err(e),
    },
    Err(_) => return Err(Error::new(ErrorKind::TimedOut, "failed connect to server")),
  }

  let elapsed_time = start_time.elapsed();

  Ok(elapsed_time.as_millis())
}

/// Состояние пинга
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PingState {
  Fast,
  Normal,
  Slow,
  Slowest,
}

/// Функция получения состояния пинга
pub fn get_ping_state(ping: impl Into<u128>) -> PingState {
  match ping.into() {
    (0..=65) => PingState::Fast,
    (66..=180) => PingState::Normal,
    (181..=500) => PingState::Slow,
    _ => PingState::Slowest,
  }
}

#[cfg(test)]
mod tests {
  use crate::bot::ping::{get_ping_state, ping_server, ping_server_with_proxy};
  use crate::proxy::Proxy;

  #[tokio::test]
  async fn test_ping_server() -> std::io::Result<()> {
    let ping = ping_server("localhost", 25565).await?;

    println!("Пинг (без прокси): {}ms", ping);
    println!("Состояние пинга: {:?}", get_ping_state(ping));

    Ok(())
  }

  #[tokio::test]
  async fn test_ping_server_with_proxy() -> std::io::Result<()> {
    let proxy = Proxy::from("socks4://68.71.242.118:4145");
    let ping = ping_server_with_proxy("hub.holyworld.ru", 25565, &proxy).await?;

    println!("Пинг (с прокси): {}ms", ping);
    println!("Состояние пинга: {:?}", get_ping_state(ping));

    Ok(())
  }
}
