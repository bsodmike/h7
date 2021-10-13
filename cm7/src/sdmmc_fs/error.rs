use {
    embedded_sdmmc::BlockDevice,
    stm32h7xx_hal::{
        pac::SDMMC2,
        sdmmc::{Sdmmc, SdmmcBlockDevice},
    },
};

pub type SdmmcFsErrorInternal = <SdmmcBlockDevice<Sdmmc<SDMMC2>> as BlockDevice>::Error;

pub enum SdmmcFsError {
    NotFound,
    BufferTooSmall,
    AlreadyMounted,
    NotMounted,
    Internal(SdmmcFsErrorInternal),
}

// impl From<SdmmcFsErrorInternal> for SdmmcFsError {
//     fn from(err: SdmmcFsErrorInternal) -> Self {
//         Self::Internal(err)
//     }
// }
