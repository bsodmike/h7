/// The Write in Progress (WIP) bit
pub const WIP: u8 = 0x01 << 0;
/// The Write Enable Latch (WEL) bit
pub const WEL: u8 = 0x01 << 1;
/// The Block Protect BP0 bit
pub const BP0: u8 = 0x01 << 2;
/// The Block Protect BP1 bit
pub const BP1: u8 = 0x01 << 3;
/// The Block Protect BP2 bit
pub const BP2: u8 = 0x01 << 4;
/// The Block Protect BP3 bit
pub const BP3: u8 = 0x01 << 5;
/// The Quad Enable bit
pub const QE: u8 = 0x01 << 6;
/// The Status Register Write Disable (SRWD) bit
pub const SRWD: u8 = 0x01 << 7;

/// The protect level 0
pub const PROTECT_LEVEL_0: u8 = 0x00 << 2;
/// The protect level 1
pub const PROTECT_LEVEL_1: u8 = 0x01 << 2;
/// The protect level 2
pub const PROTECT_LEVEL_2: u8 = 0x02 << 2;
/// The protect level 3
pub const PROTECT_LEVEL_3: u8 = 0x03 << 2;
/// The protect level mask
pub const PROTECT_LEVEL_MASK: u8 = 0x03 << 2;
