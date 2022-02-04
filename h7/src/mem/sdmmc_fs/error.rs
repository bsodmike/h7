use {
    embedded_sdmmc::BlockDevice,
    stm32h7xx_hal::{
        pac::SDMMC2,
        sdmmc::{Error, Sdmmc, SdmmcBlockDevice},
    },
};

pub type SdmmcFsErrorInternal = <SdmmcBlockDevice<Sdmmc<SDMMC2>> as BlockDevice>::Error;

pub enum SdmmcFsError {
    NotFound,
    BufferTooSmall,
    AlreadyMounted,
    NotMounted,
    Sdmmc(embedded_sdmmc::Error<stm32h7xx_hal::sdmmc::Error>),
    HalSdmmc(Error),
}

impl From<Error> for SdmmcFsError {
    fn from(err: Error) -> Self {
        Self::HalSdmmc(err)
    }
}

impl core::fmt::Display for SdmmcFsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Not Found"),
            Self::BufferTooSmall => write!(f, "Buffer Too Small"),
            Self::AlreadyMounted => write!(f, "Already mounted"),
            Self::NotMounted => write!(f, "Not Mounted"),
            Self::Sdmmc(e) => write!(f, "Sdmmc: {:?}", e),
            Self::HalSdmmc(e) => write!(f, "HalSdmmc: {:?}", e),
        }
    }
}

impl From<embedded_sdmmc::Error<stm32h7xx_hal::sdmmc::Error>> for SdmmcFsError {
    fn from(err: embedded_sdmmc::Error<stm32h7xx_hal::sdmmc::Error>) -> Self {
        Self::Sdmmc(err)
    }
}

// impl<E, B: BlockDevice<Error = E>> From<E> for SdmmcFsError {
//     fn from(err: SdmmcFsErrorInternal) -> Self {
//         // Self::Internal(err)
//         todo!()
//     }
// }
