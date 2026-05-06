use nurtex_codec::Buffer;

/// Событие игры
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum GameEvent {
  NoRespawnBlockAvailable,
  BeginRaining,
  EndRaining,
  ChangeGameMode,
  WinGame,
  DemoEvent,
  ArrowHitPlayer,
  RainLevelChange,
  ThunderLevelChange,
  PlayPufferfishStingSound,
  PlayElderGuardianMobAppearance,
  EnableRespawnScreen,
  LimitedCrafting,
  StartWaitingForLevelChunks,
}

impl Buffer for GameEvent {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = u8::read_buf(buffer)?;

    match id {
      0 => Some(Self::NoRespawnBlockAvailable),
      1 => Some(Self::BeginRaining),
      2 => Some(Self::EndRaining),
      3 => Some(Self::ChangeGameMode),
      4 => Some(Self::WinGame),
      5 => Some(Self::DemoEvent),
      6 => Some(Self::ArrowHitPlayer),
      7 => Some(Self::RainLevelChange),
      8 => Some(Self::ThunderLevelChange),
      9 => Some(Self::PlayPufferfishStingSound),
      10 => Some(Self::PlayElderGuardianMobAppearance),
      11 => Some(Self::EnableRespawnScreen),
      12 => Some(Self::LimitedCrafting),
      13 => Some(Self::StartWaitingForLevelChunks),
      _ => None,
    }
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::NoRespawnBlockAvailable => 0,
      Self::BeginRaining => 1,
      Self::EndRaining => 2,
      Self::ChangeGameMode => 3,
      Self::WinGame => 4,
      Self::DemoEvent => 5,
      Self::ArrowHitPlayer => 6,
      Self::RainLevelChange => 7,
      Self::ThunderLevelChange => 8,
      Self::PlayPufferfishStingSound => 9,
      Self::PlayElderGuardianMobAppearance => 10,
      Self::EnableRespawnScreen => 11,
      Self::LimitedCrafting => 12,
      Self::StartWaitingForLevelChunks => 13,
    };

    (id as u8).write_buf(buffer)?;

    Ok(())
  }
}
