// Array access

/// normal read
pub const READ: u8 = 0x03;

/// fast read data
pub const FAST_READ: u8 = 0x0b;

/// 2 x I/O read command
pub const READ_2: u8 = 0xbb;

/// 1I 2O read
pub const DREAD: u8 = 0x3b;

/// 4 I/O read
pub const READ_4: u8 = 0xeb;

/// 1I 4O read
pub const QREAD: u8 = 0x6b;

/// page program
pub const PP: u8 = 0x02;

/// quad page program
pub const PP_4: u8 = 0x38;

/// sector erase
pub const SE: u8 = 0x20;

/// block erase 32KB
pub const BE_32: u8 = 0x52;

/// block erase 64KB
pub const BE_64: u8 = 0xd8;

/// chip erase
pub const CE_60: u8 = 0x60;

/// chip erase
pub const CE_C7: u8 = 0xc7;

// Device operation section

/// write enable
pub const WREN: u8 = 0x06;

/// write disable
pub const WRDI: u8 = 0x04;

/// Write Protect Selection
pub const WPSEL: u8 = 0x68;

/// Enable QPI
pub const EQIO: u8 = 0x35;

/// Reset QPI
pub const RSTQIO: u8 = 0xf5;

/// Suspends Program/Erase
pub const PGM_SUSPEND_75: u8 = 0x75;

/// Suspends Program/Erase
pub const PGM_SUSPEND_B0: u8 = 0xb0;

/// Suspends Program/Erase
pub const ERS_SUSPEND_75: u8 = 0x75;

/// Suspends Program/Erase
pub const ERS_SUSPEND_B0: u8 = 0xb0;

/// Resumes Program/Erase
pub const PGM_RESUME_7A: u8 = 0x7a;

/// Resumes Program/Erase
pub const PGM_RESUME_30: u8 = 0x30;

/// Resumes Program/Erase
pub const ERS_RESUME_7A: u8 = 0x7a;

/// Resumes Program/Erase
pub const ERS_RESUME_30: u8 = 0x30;

/// Deep power down
pub const DP: u8 = 0xb9;

/// Release from deep power down
pub const RDP: u8 = 0xab;

/// No Operation
pub const NOP: u8 = 0x00;

/// Reset Enable
pub const RSTEN: u8 = 0x66;

/// Reset Memory
pub const RST: u8 = 0x99;

/// gang block lock
pub const GBLK: u8 = 0x7e;

/// gang block unlock
pub const GBULK: u8 = 0x98;

/// factory mode enable
pub const FMEN: u8 = 0x41;

// Register access section

/// read identification
pub const RDID: u8 = 0x9f;

/// read electronic ID
pub const RES: u8 = 0xab;

/// read electronic manufacturer & device ID
pub const REMS: u8 = 0x90;

/// QPI ID Read
pub const QPIID: u8 = 0xaf;

/// Read SFDP Table
pub const RDSFDP: u8 = 0x5a;

/// read status register
pub const RDSR: u8 = 0x05;

/// read configuration register
pub const RDCR: u8 = 0x15;

/// write status/configuration register
pub const WRSR: u8 = 0x01;

/// write status/configuration register
pub const WRCR: u8 = 0x01;

/// read security register
pub const RDSCUR: u8 = 0x2b;

/// write security register
pub const WRSCUR: u8 = 0x2f;

/// Set Burst Length
pub const SBL: u8 = 0xc0;

/// enter secured OTP
pub const ENSO: u8 = 0xb1;

/// exit secured OTP
pub const EXSO: u8 = 0xc1;

/// write Lock register
pub const WRLR: u8 = 0x2c;

/// read Lock register
pub const RDLR: u8 = 0x2d;

/// SPB bit program
pub const WRSPB: u8 = 0xe3;

/// all SPB bit erase
pub const ESSPB: u8 = 0xe4;

/// read SPB status
pub const RDSPB: u8 = 0xe2;
