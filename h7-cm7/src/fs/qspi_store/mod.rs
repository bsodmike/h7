use {
    core::cell::RefCell,
    critical_section::Mutex,
    mx25l::Mx25L,
    stm32h7xx_hal::{
        gpio::{gpiog::PG6, Output, PushPull},
        pac::QUADSPI,
        xspi::{Qspi, QspiError},
    },
};

#[allow(dead_code)]
pub mod mx25l;

pub const QSPI_FLASH_SIZE: usize = 16 * 1024 * 1024;
pub static QSPI_STORE: Mutex<RefCell<Option<NorFlash>>> = Mutex::new(RefCell::new(None));

pub struct NorFlash {
    mx25l: Mx25L<PG6<Output<PushPull>>>,
}

impl NorFlash {
    pub fn new(qspi: Qspi<QUADSPI>, cs: PG6<Output<PushPull>>) -> Self {
        Self {
            mx25l: Mx25L::new(qspi, cs),
        }
    }

    pub fn init(&mut self) -> Result<(), QspiError> {
        self.mx25l.init()
    }

    // pub fn inner(&mut self) -> &mut Mx25L<PG6<Output<PushPull>>> {
    //     &mut self.mx25l
    // }

    // pub fn free(self) -> Mx25L<PG6<Output<PushPull>>> {
    //     self.mx25l
    // }
}

impl core::ops::Deref for NorFlash {
    type Target = Mx25L<PG6<Output<PushPull>>>;

    fn deref(&self) -> &Self::Target {
        &self.mx25l
    }
}

impl core::ops::DerefMut for NorFlash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mx25l
    }
}
