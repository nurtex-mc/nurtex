use nurtex::{Bot, Cluster, JoinDelay};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём кластер
  let mut cluster = Cluster::create();

  // Создаём 3 роя
  for s_ind in 0..3 {
    let mut bots = Vec::new();

    // Создаём 2 бота
    for b_ind in 0..2 {
      // Создаём бота и добавляем его в список
      bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
    }

    cluster.add_swarm(bots, JoinDelay::fixed(1000), "localhost", 25565);
  }

  // Запускаем кластер и ожидаем завершения всех хэндлов
  cluster.launch_and_wait().await
}
