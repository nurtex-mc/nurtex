use std::time::Duration;

use nurtex::{Bot, Cluster, JoinDelay};

#[tokio::main]
async fn main() {
  // Создаём кластер
  let mut cluster = Cluster::create();

  for i in 0..3 {
    // Добавляем рои в кластер.
    // Важно: Нужно добавлять рои каждый раз после `shutdown` (выключения кластера)
    for s_ind in 0..6 {
      let mut bots = Vec::new();

      for b_ind in 0..3 {
        bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
      }

      cluster.add_swarm(bots, JoinDelay::fixed(500), "localhost", 25565);
    }

    // Запускаем кластер
    cluster.launch();

    // Ждём немножко
    tokio::time::sleep(Duration::from_secs(6)).await;

    // Отключаем и очищаем кластер
    cluster.shutdown().await;

    // Ждём перед следующим запуском (за исключением последнего запуска)
    if i != 2 {
      tokio::time::sleep(Duration::from_secs(3)).await;
    }
  }
}
