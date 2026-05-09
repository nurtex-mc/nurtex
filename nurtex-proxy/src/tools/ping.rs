use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Instant;

use hashbrown::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::Proxy;

/// Структура информации о пинге
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PingInfo {
  pub pinged_services: HashMap<String, u64>,
  pub average_ping: Option<u64>,
}

/// Вспомогательная функция получения пингуемых сервисов по умолчанию
fn default_pinged_services() -> Vec<(String, u16)> {
  vec![
    ("cloudflare.com".to_string(), 80),
    ("facebook.com".to_string(), 80),
    ("yandex.ru".to_string(), 80),
    ("youtube.com".to_string(), 80),
    ("github.com".to_string(), 80),
    ("reddit.com".to_string(), 80),
  ]
}

/// Функция пингования прокси
pub async fn ping_proxy(proxy: &Proxy, pinged_services: Option<Vec<(String, u16)>>) -> PingInfo {
  let mut ping_info = PingInfo {
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  if !proxy.is_available().await {
    return ping_info;
  }

  let mut total_pinged_services = 0;
  let mut total_ping = 0;

  let ps = if let Some(services) = pinged_services { services } else { default_pinged_services() };

  for (service_host, service_port) in ps {
    let start_time = Instant::now();

    match proxy.connect(&service_host, service_port).await {
      Ok(mut s) => {
        let _ = s.shutdown().await;

        let ping = start_time.elapsed().as_millis();

        total_pinged_services += 1;
        total_ping += ping as u64;

        ping_info.pinged_services.insert(service_host, ping as u64);
      }
      Err(_) => {}
    }
  }

  ping_info.average_ping = Some(total_ping / total_pinged_services as u64);

  ping_info
}

/// Функция параллельного пингования прокси
pub async fn ping_proxy_parallel(proxy: Arc<Proxy>, pinged_services: Option<Vec<(String, u16)>>) -> PingInfo {
  let ping_info = PingInfo {
    pinged_services: HashMap::new(),
    average_ping: None,
  };

  let ping_info_mutex = Arc::new(Mutex::new(ping_info));

  if !proxy.is_available().await {
    return ping_info_mutex.lock().await.clone();
  }

  let total_pinged_services = Arc::new(AtomicU8::new(0));
  let total_ping = Arc::new(AtomicU64::new(0));

  let mut handles = Vec::new();

  let ps = if let Some(services) = pinged_services { services } else { default_pinged_services() };

  for (service_host, service_port) in ps {
    let proxy_clone = Arc::clone(&proxy);
    let ping_info_mutex_clone = Arc::clone(&ping_info_mutex);
    let total_pinged_services_clone = Arc::clone(&total_pinged_services);
    let total_ping_clone = Arc::clone(&total_ping);

    let handle = tokio::spawn(async move {
      let start_time = Instant::now();

      match proxy_clone.connect(&service_host, service_port).await {
        Ok(mut s) => {
          let _ = s.shutdown().await;

          let ping = start_time.elapsed().as_millis();

          total_pinged_services_clone.fetch_add(1, Ordering::SeqCst);
          total_ping_clone.fetch_add(ping as u64, Ordering::SeqCst);

          ping_info_mutex_clone.lock().await.pinged_services.insert(service_host, ping as u64);
        }
        Err(_) => {}
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    let _ = handle.await;
  }

  let average_ping = total_ping.load(Ordering::SeqCst) / total_pinged_services.load(Ordering::SeqCst) as u64;
  ping_info_mutex.lock().await.average_ping = Some(average_ping);

  ping_info_mutex.lock().await.clone()
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::tools::{ping_proxy, ping_proxy_parallel};
  use crate::{Proxy, ProxyType};

  #[tokio::test]
  async fn test_ping_proxy() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    let result = ping_proxy(&proxy, None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг прокси: {}ms", average_ping);
    }
  }

  #[tokio::test]
  async fn test_ping_proxy_parallel() {
    let proxy = Proxy::new("98.175.31.222:4145", ProxyType::Socks5);

    let result = ping_proxy_parallel(Arc::new(proxy), None).await;

    for (name, ping) in result.pinged_services {
      println!("Пинг {}: {}ms", name, ping);
    }

    println!("===============================");

    if let Some(average_ping) = result.average_ping {
      println!("Средний пинг прокси: {}ms", average_ping);
    }
  }
}
