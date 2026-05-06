use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() {
  // Создаём список ботов
  let mut bots = Vec::new();

  // Добавляем ботов в наш список
  for i in 0..6 {
    bots.push(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Создаём рой и запускаем его на сервер
  Swarm::create()
    .with_bots(bots)
    .with_join_delay(JoinDelay::fixed(500))
    .bind("localhost", 25565)
    .launch_and_wait()
    .await
}
