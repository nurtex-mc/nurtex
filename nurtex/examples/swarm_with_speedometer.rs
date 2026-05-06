use std::sync::Arc;

use nurtex::bot::Bot;
use nurtex::swarm::{JoinDelay, Speedometer, SpeedometerEvent, Swarm};

#[tokio::main]
async fn main() {
  // Создаём спидометр
  let speedometer = Arc::new(Speedometer::new(100));

  // Создаём рой
  let mut swarm = Swarm::create().with_join_delay(JoinDelay::regressive_linear(5000, 50)).bind("localhost", 25565);

  // Добавляем ботов в рой
  for i in 0..50 {
    // Создаём бота со спидометром
    let speedometer = Arc::clone(&speedometer);
    let bot = Bot::create_with_speedometer(format!("nurtex_bot_{}", i), speedometer);

    // Добавляем бота в рой
    swarm.add_bot(bot);
  }

  // Запускаем рой на сервер
  swarm.quiet_launch();

  // Подписываемся на события спидометра
  let mut speedometer_rx = speedometer.subscribe();

  // Создаём бесконечный цикл
  loop {
    if let Ok(event) = speedometer_rx.recv().await {
      match event {
        SpeedometerEvent::TimerTick { speed, boost } => {
          // Обрабатываем тик таймера
          println!("Фиксированная скорость: {} b/s (буст: {})", speed, boost);
        }
        SpeedometerEvent::UpdatePeakSpeed(speed) => {
          // Обрабатываем пиковую скорость
          println!("Новая пиковая скорость: {} b/s", speed);
        }
        _ => {}
      }
    }
  }
}
