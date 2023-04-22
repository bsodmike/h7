use {
    embedded_hal::digital::v2::OutputPin,
    stm32h7xx_hal::{
        pac::QUADSPI,
        rcc,
        xspi::{Qspi, QspiError, QspiMode, QspiWord},
    },
};

/// RDID (Read Identification)
const MX25L_CMD_RDID: u8 = 0x9F;
/// RES (Read Electronic ID)
const MX25L_CMD_RES: u8 = 0xAB;
/// REMS (Read Electronic & Device ID)
const MX25L_CMD_REMS: u8 = 0x90;

/// WRSR (Write Status Register)
const MX25L_CMD_WRSR: u8 = 0x01;
/// RDSR (Read Status Register)
const MX25L_CMD_RDSR: u8 = 0x05;

/// READ (1 x I/O)
const MX25L_CMD_READ: u8 = 0x03;
/// FAST READ (Fast read data)
const MX25L_CMD_FASTREAD: u8 = 0x0B;
/// DREAD (1In/2 Out fast read)
const MX25L_CMD_DREAD: u8 = 0x3B;
const MX25L_CMD_READ_QUAD: u8 = 0x6B;

/// WREN (Write Enable)
const MX25L_CMD_WREN: u8 = 0x06;
/// WRDI (Write Disable)
const MX25L_CMD_WRDI: u8 = 0x04;
/// PP (page program)
const MX25L_CMD_PP: u8 = 0x02;

/// SE (Sector Erase)
const MX25L_CMD_SE: u8 = 0x20;
/// BE (Block Erase)
const MX25L_CMD_BE: u8 = 0xD8;
/// CE (Chip Erase) hex code: 60 or C7
const MX25L_CMD_CE: u8 = 0x60;

/// DP (Deep Power Down)
const MX25L_CMD_DP: u8 = 0xB9;
/// RDP (Release form Deep Power Down)
const MX25L_CMD_RDP: u8 = 0xAB;

/// The Write in Progress (WIP) bit
const MX25L_STATUS_WIP: u8 = 0x01 << 0;
/// The Write Enable Latch (WEL) bit
const MX25L_STATUS_WEL: u8 = 0x01 << 1;
/// The Block Protect BP0 bit
const MX25L_STATUS_BP0: u8 = 0x01 << 2;
/// The Block Protect BP1 bit
const MX25L_STATUS_BP1: u8 = 0x01 << 3;
/// The Status Register Write Disable (SRWD) bit
const MX25L_STATUS_SRWD: u8 = 0x01 << 7;

/// The protect level 0
const MX25L_STATUS_PROTECT_LEVEL_0: u8 = 0x00 << 2;
/// The protect level 1
const MX25L_STATUS_PROTECT_LEVEL_1: u8 = 0x01 << 2;
/// The protect level 2
const MX25L_STATUS_PROTECT_LEVEL_2: u8 = 0x02 << 2;
/// The protect level 3
const MX25L_STATUS_PROTECT_LEVEL_3: u8 = 0x03 << 2;
/// The protect level mask
const MX25L_STATUS_PROTECT_LEVEL_MASK: u8 = 0x03 << 2;

pub struct Mx25L<CS: OutputPin> {
    qspi: Qspi<QUADSPI>,
    cs: CS,
}

impl<CS: OutputPin> Mx25L<CS> {
    pub fn new(qspi: Qspi<QUADSPI>, cs: CS) -> Self {
        Self { qspi, cs }
    }

    pub fn init(&mut self) -> Result<(), QspiError> {
        self.qspi.configure_mode(QspiMode::OneBit)?;
        // self.write_extended(QspiWord::U8(0x41), QspiWord::None, QspiWord::None, &[])?;
        self.reset()?;
        self.exit_deep_sleep()?;
        self.write_status()?;

        Ok(())
    }

    pub fn reaq_identification(&mut self) -> Result<[u8; 3], QspiError> {
        let mut id = [0u8; 3];
        self.read_extended(
            QspiWord::U8(0x9f),
            QspiWord::None,
            QspiWord::None,
            0,
            &mut id,
        )?;
        Ok(id)
    }

    pub fn read_status(&mut self) -> Result<u8, QspiError> {
        let mut status = [0u8; 1];
        self.read_extended(
            QspiWord::U8(MX25L_CMD_RDSR),
            QspiWord::None,
            QspiWord::None,
            0,
            &mut status,
        )?;

        Ok(status[0])
    }

    pub fn read_config(&mut self) -> Result<u8, QspiError> {
        let mut status = [0u8; 1];
        self.read_extended(
            QspiWord::U8(0x15),
            QspiWord::None,
            QspiWord::None,
            0,
            &mut status,
        )?;

        Ok(status[0])
    }

    pub fn write_status(&mut self) -> Result<(), QspiError> {
        self.enable_write()?;
        self.write_extended(
            QspiWord::U8(MX25L_CMD_WRSR),
            QspiWord::None,
            QspiWord::None,
            &[0b0000_0010],
        )?;
        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
        Ok(())
    }

    pub fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), QspiError> {
        self.read_extended(
            QspiWord::U8(MX25L_CMD_READ),
            QspiWord::U24(address),
            QspiWord::None,
            0,
            data,
        )?;
        Ok(())
    }

    pub fn write(&mut self, address: u32, data: &[u8]) -> Result<(), QspiError> {
        // self.enable_write()?;
        // self.write_extended(
        //     QspiWord::U8(MX25L_CMD_BE),
        //     QspiWord::U24(address),
        //     QspiWord::None,
        //     &[],
        // )?;
        self.enable_write()?;
        self.write_extended(
            QspiWord::U8(MX25L_CMD_PP),
            QspiWord::U24(address),
            QspiWord::None,
            data,
        )?;
        Ok(())
    }

    pub fn chip_erase(&mut self) -> Result<(), QspiError> {
        self.enable_write()?;
        self.write_extended(
            QspiWord::U8(MX25L_CMD_CE),
            QspiWord::None,
            QspiWord::None,
            &[],
        )
    }

    pub fn enter_deep_sleep(&mut self) -> Result<(), QspiError> {
        self.write_extended(
            QspiWord::U8(MX25L_CMD_DP),
            QspiWord::None,
            QspiWord::None,
            &[],
        )?;
        Ok(())
    }

    pub fn exit_deep_sleep(&mut self) -> Result<(), QspiError> {
        self.write_extended(
            QspiWord::U8(MX25L_CMD_RDP),
            QspiWord::None,
            QspiWord::None,
            &[],
        )?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), QspiError> {
        self.qspi
            .write_extended(QspiWord::U8(0x66), QspiWord::None, QspiWord::None, &[])?;
        self.chip_deselect();
        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
        self.chip_select();
        self.qspi
            .write_extended(QspiWord::U8(0x99), QspiWord::None, QspiWord::None, &[])?;
        self.chip_deselect();
        self.write_status()?;

        Ok(())
    }

    pub fn free(self) -> (QUADSPI, rcc::rec::Qspi) {
        self.qspi.free()
    }

    // --------------------------------------------------------

    #[allow(clippy::wrong_self_convention)]
    fn is_busy(&mut self) -> Result<bool, QspiError> {
        Ok((self.read_status()? & MX25L_STATUS_WIP) != 0)
    }

    fn enable_write(&mut self) -> Result<(), QspiError> {
        self.write_extended(
            QspiWord::U8(MX25L_CMD_WREN),
            QspiWord::None,
            QspiWord::None,
            &[],
        )?;
        Ok(())
    }

    fn chip_select(&mut self) {
        let _ = self.cs.set_low();
    }

    fn chip_deselect(&mut self) {
        let _ = self.cs.set_high();
    }

    fn write_extended(
        &mut self,
        instruction: QspiWord,
        address: QspiWord,
        alternate_bytes: QspiWord,
        data: &[u8],
    ) -> Result<(), QspiError> {
        self.chip_select();
        self.qspi
            .write_extended(instruction, address, alternate_bytes, data)?;
        self.chip_deselect();
        while self.is_busy()? {}
        Ok(())
    }

    fn read_extended(
        &mut self,
        instruction: QspiWord,
        address: QspiWord,
        alternate_bytes: QspiWord,
        dummy_cycles: u8,
        dest: &mut [u8],
    ) -> Result<(), QspiError> {
        self.chip_select();
        self.qspi
            .read_extended(instruction, address, alternate_bytes, dummy_cycles, dest)?;
        self.chip_deselect();
        Ok(())
    }
}
