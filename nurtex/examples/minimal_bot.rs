#[tokio::main]
async fn main() -> std::io::Result<()> {
  // Создаём бота и сразу подключаем его к серверу
  nurtex::Bot::create("nurtex_bot").connect_with_handle("localhost", 25565).await?
}
