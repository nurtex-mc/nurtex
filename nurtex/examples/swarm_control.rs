use std::time::Duration;

use nurtex::bot::{Bot, BotChatExt};
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой
  let mut swarm = Swarm::create().with_join_delay(JoinDelay::fixed(500)).bind("localhost", 25565);

  // Добавляем ботов в рой
  for i in 0..6 {
    swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
  }

  // Запускаем рой на сервер
  swarm.launch().await;

  // Ждём немножко
  tokio::time::sleep(Duration::from_secs(2)).await;

  // Параллельно проходимся по всем ботам из роя
  swarm.for_each_parallel(async |bot| {
    // Отправляем сообщение в чат
    bot.chat_message(format!("Мой юзернейм: {}", bot.username())).await
  });

  // Ждём 10 секунд и завершаем процесс
  tokio::time::sleep(Duration::from_secs(10)).await;

  Ok(())
}
