use {
    core::cell::RefCell,
    cortex_m::interrupt::{free as interrupt_free, Mutex},
    embedded_sdmmc::BlockDevice,
    mx25l::Mx25L,
    stm32h7xx_hal::{
        gpio::{gpiog::PG6, Output, PushPull},
        pac::QUADSPI,
        xspi::{Qspi, QspiError},
    },
};

pub mod mx25l;

pub const QSPI_FLASH_SIZE: usize = 16 * 1024 * 1024;
pub static QSPI_FS: Mutex<RefCell<Option<QspiFs>>> = Mutex::new(RefCell::new(None));

pub struct QspiFs {
    mx25l: Mx25L<PG6<Output<PushPull>>>,
}

impl QspiFs {
    pub fn new(qspi: Qspi<QUADSPI>, cs: PG6<Output<PushPull>>) -> Self {
        Self {
            mx25l: Mx25L::new(qspi, cs),
        }
    }

    pub fn init(&mut self) -> Result<(), QspiError> {
        self.mx25l.init()
    }

    pub fn inner(&mut self) -> &mut Mx25L<PG6<Output<PushPull>>> {
        &mut self.mx25l
    }

    pub fn free(self) -> Mx25L<PG6<Output<PushPull>>> {
        self.mx25l
    }
}

// TODO:
// impl BlockDevice for QspiFs {
//     type Error = QspiError;

//     fn read(
//         &self,
//         blocks: &mut [embedded_sdmmc::Block],
//         start_block_idx: embedded_sdmmc::BlockIdx,
//         reason: &str,
//     ) -> Result<(), Self::Error> {
//         todo!()
//     }

//     fn write(
//         &self,
//         blocks: &[embedded_sdmmc::Block],
//         start_block_idx: embedded_sdmmc::BlockIdx,
//     ) -> Result<(), Self::Error> {
//         todo!()
//     }

//     fn num_blocks(&self) -> Result<embedded_sdmmc::BlockCount, Self::Error> {
//         todo!()
//     }
// }
