/// Макрос для объявления модуля пакетов
macro_rules! declare_packet_module {
  ($name:ident) => {
    pub mod $name {
      mod enumeration;
      mod packets;

      pub use enumeration::*;
      pub use packets::*;
    }
  };
}

declare_packet_module!(configuration);
declare_packet_module!(handshake);
declare_packet_module!(login);
declare_packet_module!(play);
declare_packet_module!(status);
