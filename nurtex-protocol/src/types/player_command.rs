use nurtex_codec::Buffer;
use nurtex_codec::types::variable::VarI32;

/// Команда игрока
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PlayerCommand {
  LeaveBed,
  StartSprinting,
  StopSprinting,
  StartJumpWithHorse,
  StopJumpWithHorse,
  OpenVehicleInventory,
  StartElytraFlying,
}

impl Buffer for PlayerCommand {
  fn read_buf(buffer: &mut std::io::Cursor<&[u8]>) -> Option<Self> {
    let id = i32::read_var(buffer)?;

    Some(match id {
      0 => Self::LeaveBed,
      1 => Self::StartSprinting,
      2 => Self::StopSprinting,
      3 => Self::StartJumpWithHorse,
      4 => Self::StopJumpWithHorse,
      5 => Self::OpenVehicleInventory,
      6 => Self::StartElytraFlying,
      _ => return None,
    })
  }

  fn write_buf(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let id = match self {
      Self::LeaveBed => 0,
      Self::StartSprinting => 1,
      Self::StopSprinting => 2,
      Self::StartJumpWithHorse => 3,
      Self::StopJumpWithHorse => 4,
      Self::OpenVehicleInventory => 5,
      Self::StartElytraFlying => 6,
    };

    id.write_var(buffer)?;

    Ok(())
  }
}
