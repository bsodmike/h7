use stm32h7xx_hal::sdmmc::Error;

pub enum SdmmcFsError {
    NotFound,
    // BufferTooSmall,
    AlreadyMounted,
    NotMounted,
    Sdmmc(embedded_sdmmc::Error<Error>),
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
            // Self::BufferTooSmall => write!(f, "Buffer Too Small"),
            Self::AlreadyMounted => write!(f, "Already mounted"),
            Self::NotMounted => write!(f, "Not Mounted"),
            Self::Sdmmc(e) => write!(f, "Sdmmc: {e:?}"),
            Self::HalSdmmc(e) => write!(f, "HalSdmmc: {e:?}"),
        }
    }
}

impl From<embedded_sdmmc::Error<Error>> for SdmmcFsError {
    fn from(err: embedded_sdmmc::Error<Error>) -> Self {
        Self::Sdmmc(err)
    }
}

// impl<E, B: BlockDevice<Error = E>> From<E> for SdmmcFsError {
//     fn from(err: SdmmcFsErrorInternal) -> Self {
//         // Self::Internal(err)
//         todo!()
//     }
// }
