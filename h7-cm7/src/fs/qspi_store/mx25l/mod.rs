use {
    embedded_hal::digital::v2::OutputPin,
    stm32h7xx_hal::{
        pac::QUADSPI,
        rcc,
        xspi::{Qspi, QspiError, QspiMode, QspiWord},
    },
};

pub mod cmd;
pub mod status;

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
            QspiWord::U8(cmd::RDSR),
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
            QspiWord::U8(cmd::WRSR),
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
            QspiWord::U8(cmd::READ),
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
        //     QspiWord::U8(cmd::BE_32),
        //     QspiWord::U24(address),
        //     QspiWord::None,
        //     &[],
        // )?;

        // the internal write buffer is 32 bytes
        for chunk in data.chunks(32) {
            self.enable_write()?;
            self.write_extended(
                QspiWord::U8(cmd::PP),
                QspiWord::U24(address),
                QspiWord::None,
                chunk,
            )?;
        }

        Ok(())
    }

    pub fn chip_erase(&mut self) -> Result<(), QspiError> {
        self.enable_write()?;
        self.write_extended(
            QspiWord::U8(cmd::CE_60),
            QspiWord::None,
            QspiWord::None,
            &[],
        )
    }

    pub fn enter_deep_sleep(&mut self) -> Result<(), QspiError> {
        self.write_extended(QspiWord::U8(cmd::DP), QspiWord::None, QspiWord::None, &[])?;
        Ok(())
    }

    pub fn exit_deep_sleep(&mut self) -> Result<(), QspiError> {
        self.write_extended(QspiWord::U8(cmd::RDP), QspiWord::None, QspiWord::None, &[])?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), QspiError> {
        self.qspi.write_extended(
            QspiWord::U8(cmd::RSTEN),
            QspiWord::None,
            QspiWord::None,
            &[],
        )?; // RSTEN
        self.chip_deselect();
        for _ in 0..100_000 {
            cortex_m::asm::nop();
        }
        self.chip_select();
        self.qspi
            .write_extended(QspiWord::U8(cmd::RST), QspiWord::None, QspiWord::None, &[])?; // RST
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
        Ok((self.read_status()? & status::WIP) != 0)
    }

    fn enable_write(&mut self) -> Result<(), QspiError> {
        loop {
            self.write_extended(QspiWord::U8(cmd::WREN), QspiWord::None, QspiWord::None, &[])?;
            while self.is_busy()? {
                cortex_m::asm::dsb();
            }
            if (self.read_status()? & status::WEL) == status::WEL {
                break;
            }
        }

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
        while self.is_busy()? {
            cortex_m::asm::dsb();
        }
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
