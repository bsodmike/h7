use {
    crate::time::TimeSource,
    core::cell::RefCell,
    cortex_m::interrupt::Mutex,
    embedded_hal::blocking::delay::DelayMs,
    embedded_sdmmc::{BlockDevice, Controller},
    error::*,
    stm32h7xx_hal::{
        pac::SDMMC2,
        sdmmc::{Sdmmc, SdmmcBlockDevice},
        time::Hertz,
    },
};

mod error;
mod path;

pub static SD_CARD: Mutex<RefCell<Option<SdmmcFs>>> = Mutex::new(RefCell::new(None));

enum SdmmcState {
    Controller(Controller<SdmmcBlockDevice<Sdmmc<SDMMC2>>, TimeSource>),
    Sdmmc(Sdmmc<SDMMC2>),
    MidSwap,
}

pub struct SdmmcFs {
    state: SdmmcState,
}

impl SdmmcFs {
    pub fn new(sdmmc: Sdmmc<SDMMC2>) -> Self {
        Self {
            state: SdmmcState::Sdmmc(sdmmc),
        }
    }

    pub fn is_mounted(&self) -> bool {
        match &self.state {
            SdmmcState::Controller(_) => true,
            SdmmcState::Sdmmc(_) => false,
            _ => unreachable!(),
        }
    }

    pub fn card_size(&mut self) -> Result<u64, SdmmcFsError> {
        match self.state {
            SdmmcState::Controller(ref mut c) => {
                let blocks = c.device().num_blocks()?.0;
                Ok(blocks as u64 * 512)
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }

    pub fn mount<D: DelayMs<u16>, H: Into<Hertz>>(
        &mut self,
        freq: H,
        n_retry: u8,
        mut delay: Option<(u16, &mut D)>,
    ) -> Result<(), SdmmcFsError> {
        match &mut self.state {
            SdmmcState::Controller(_) => return Err(SdmmcFsError::AlreadyMounted),
            SdmmcState::Sdmmc(sdmmc) => {
                let freq = freq.into();
                for i in (0..n_retry).rev() {
                    match sdmmc.init_card(freq) {
                        Ok(_) => {
                            // We just got here because the state is SdmmcState::Sdmmc so this should never fail
                            if let SdmmcState::Sdmmc(sd) =
                                core::mem::replace(&mut self.state, SdmmcState::MidSwap)
                            {
                                self.state = SdmmcState::Controller(Controller::new(
                                    sd.sdmmc_block_device(),
                                    TimeSource,
                                ));
                            } else {
                                unreachable!()
                            }
                            return Ok(());
                        }
                        Err(e) => {
                            if i == 0 {
                                return Err(SdmmcFsError::HalSdmmc(e));
                            } else {
                                let _ = log::info!("SD Card mount failed, retrying...");
                                if let Some((time, ref mut delay)) = delay {
                                    delay.delay_ms(time);
                                }
                                continue;
                            }
                        }
                    }
                }
            }
            SdmmcState::MidSwap => unreachable!(),
        };
        unreachable!()
    }

    /// Useless until https://github.com/stm32-rs/stm32h7xx-hal/issues/145 is fixed
    pub fn unmount(&mut self) -> Result<(), SdmmcFsError> {
        match &mut self.state {
            SdmmcState::Controller(_) => {
                if let SdmmcState::Controller(c) =
                    core::mem::replace(&mut self.state, SdmmcState::MidSwap)
                {
                    self.state = SdmmcState::Sdmmc(c.free().0.free());
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }

    pub fn read_file<P: AsRef<str>>(
        &mut self,
        path: P,
        data: &mut [u8],
    ) -> Result<usize, SdmmcFsError> {
        match &mut self.state {
            SdmmcState::Controller(controller) => {
                let mut volume = controller.get_volume(embedded_sdmmc::VolumeIdx(0))?;
                let root_dir = controller.open_root_dir(&volume)?;

                let open_result = match controller.open_file_in_dir(
                    &mut volume,
                    &root_dir,
                    path.as_ref(),
                    embedded_sdmmc::Mode::ReadOnly,
                ) {
                    Ok(mut file) => {
                        if data.len() < file.length() as usize {
                            controller.close_file(&volume, file)?;
                            Err(SdmmcFsError::BufferTooSmall)
                        } else {
                            let read_result = controller.read(&volume, &mut file, data);
                            controller.close_file(&volume, file)?;
                            read_result.map_err(SdmmcFsError::Sdmmc)
                        }
                    }
                    Err(e) => Err(SdmmcFsError::Sdmmc(e)),
                };

                controller.close_dir(&volume, root_dir);
                Ok(open_result?)
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }

    // pub fn write_file<P: AsRef<str>>(
    //     &mut self,
    //     path: P,
    //     create: bool,
    //     data: &[u8],
    // ) -> Result<(), SdmmcFsError> {
    //     match &mut self.state {
    //         SdmmcState::Controller(controller) => {
    //             todo!()
    //         }
    //         SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
    //         SdmmcState::MidSwap => unreachable!(),
    //     }
    // }

    // pub fn exists<P: AsRef<str>>(&mut self, path: P) -> Result<bool, SdmmcFsError> {
    //     match &mut self.state {
    //         SdmmcState::Controller(controller) => {
    //             todo!()
    //         }
    //         SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
    //         SdmmcState::MidSwap => unreachable!(),
    //     }
    // }

    pub fn files(
        &mut self,
        mut l: impl FnMut(&embedded_sdmmc::DirEntry),
    ) -> Result<(), SdmmcFsError> {
        match self.state {
            SdmmcState::Controller(ref mut controller) => {
                let volume = controller.get_volume(embedded_sdmmc::VolumeIdx(0))?;
                let root_dir = controller.open_root_dir(&volume)?;
                controller.iterate_dir(&volume, &root_dir, |entry| l(entry))?;
                controller.close_dir(&volume, root_dir);
                Ok(())
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }
}

// let mut sd = dp.SDMMC2.sdmmc(
//     (
//         gpiod.pd6.into_alternate_af11(),
//         gpiod.pd7.into_alternate_af11(),
//         gpiob.pb14.into_alternate_af9(),
//         gpiob.pb15.into_alternate_af9(),
//         gpiob.pb3.into_alternate_af9(),
//         gpiob.pb4.into_alternate_af9(),
//     ),
//     ccdr.peripheral.SDMMC2,
//     &ccdr.clocks,
// );

// // sd.init_card(25.mhz()).unwrap();
// // Loop until we have a card
// loop {
//     match sd.init_card(2.mhz()) {
//         Ok(_) => break,
//         Err(err) => {
//             log::info!("Init err: {:?}", err);
//         }
//     }

//     log::info!("Waiting for card...");

//     delay.delay_ms(1000u32);
// }

// let mut sd_fatfs = embedded_sdmmc::Controller::new(sd.sdmmc_block_device(), time::TimeSource);
// let sd_fatfs_volume = sd_fatfs.get_volume(embedded_sdmmc::VolumeIdx(0)).unwrap();
// let sd_fatfs_root_dir = sd_fatfs.open_root_dir(&sd_fatfs_volume).unwrap();
// sd_fatfs
//     .iterate_dir(&sd_fatfs_volume, &sd_fatfs_root_dir, |entry| {
//         log::info!("{:?}", entry);
//     })
//     .unwrap();
// sd_fatfs.close_dir(&sd_fatfs_volume, sd_fatfs_root_dir);
