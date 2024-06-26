use enum_utils::TryFromRepr;

#[derive(Debug, TryFromRepr, PartialEq, Eq, uniffi::Enum)]
#[repr(u8)]
pub enum Status {
    Normal = 0x00,
    Busy = 0x01,
    Stopped = 0x40,
    Breakpoint = 0x41,
    Memfault = 0x43,
    Finished = 0x44,
    Running = 0x80,
    RunningSwi = 0x81,
    Stepping = 0x82,
    Broken = 0x30,
}

#[derive(Debug, uniffi::Record)]
pub struct BoardState {
    pub status: Status,
    pub steps_remaining: u32,
    pub steps_since_reset: u32,
}
