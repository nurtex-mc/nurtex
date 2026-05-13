use std::time::Duration;

use nurtex::{Bot, BotChatExt, Cluster, JoinDelay};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём кластер
  let mut cluster = Cluster::create();

  for s_ind in 0..3 {
    // Создаём список ботов
    let mut bots = Vec::new();

    for b_ind in 0..2 {
      // Создаём бота и добавляем его в список
      bots.push(Bot::create(format!("nurtex_{}_{}", s_ind, b_ind)));
    }

    // Добавляем рой в кластер
    cluster.add_swarm(bots, JoinDelay::fixed(1000), "localhost", 25565);
  }

  // Запускаем кластер
  cluster.launch();

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(5)).await;

  // Проходимся параллельно по всем ботам из всех роев
  cluster.for_each_bots_parallel(async |bot| {
    // Отправляем сообщение в чат
    bot.chat_message(format!("Привет, я {}!", bot.username())).await
  });

  // Вновь ждём немножко
  tokio::time::sleep(Duration::from_secs(5)).await;

  Ok(())
}
