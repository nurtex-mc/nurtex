use std::time::Duration;

use nurtex::bot::{Bot, BotChatExt};
use nurtex::swarm::{JoinDelay, Swarm};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём рой
  let mut swarm = Swarm::create().with_join_delay(JoinDelay::fixed(200)).bind("localhost", 25565);

  for i in 0..3 {
    // Добавляем ботов в рой.
    // Важно: Нужно добавлять ботов каждый раз после `shutdown` (выключения роя)
    for i in 0..6 {
      swarm.add_bot(Bot::create(format!("nurtex_bot_{}", i)));
    }

    // Запускаем рой на сервер
    swarm.launch().await;

    // Ждём немножко
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Параллельно проходимся по всем ботам из роя
    swarm.for_each_parallel(async |bot| {
      // Отправляем сообщение в чат
      bot.chat_message("Привет, мир!").await
    });

    // Ждём немножко
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Отключаем и очищаем рой
    swarm.shutdown().await?;

    // Ждём перед следующим запуском (за исключением последнего запуска)
    if i != 2 {
      tokio::time::sleep(Duration::from_secs(2)).await;
    }
  }

  Ok(())
}
