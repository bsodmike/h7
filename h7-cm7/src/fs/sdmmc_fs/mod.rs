use {
    super::path::Path,
    crate::time::TimeSource,
    core::{cell::RefCell, fmt},
    critical_section::Mutex,
    embedded_hal::blocking::delay::DelayMs,
    embedded_sdmmc::{
        BlockDevice, Controller, DirEntry, Directory, File, Mode as FileOpenMode, Volume, VolumeIdx,
    },
    error::*,
    stm32h7xx_hal::{
        pac::SDMMC2,
        sdmmc::{SdCard, Sdmmc, SdmmcBlockDevice},
        time::Hertz,
    },
};

mod error;

const H7_MAX_OPEN_DIRS: usize = 4;
const H7_MAX_OPEN_FILES: usize = 4;

pub static SD_CARD: Mutex<RefCell<Option<SdmmcFs<H7_MAX_OPEN_DIRS, H7_MAX_OPEN_FILES>>>> =
    Mutex::new(RefCell::new(None));

type H7Sdmmc = Sdmmc<SDMMC2, SdCard>;
type H7SdmmcBlockDev = SdmmcBlockDevice<H7Sdmmc>;

enum SdmmcState<const MAX_OPEN_DIRS: usize, const MAX_OPEN_FILES: usize> {
    Controller(Controller<H7SdmmcBlockDev, TimeSource, MAX_OPEN_DIRS, MAX_OPEN_FILES>),
    Sdmmc(H7Sdmmc),
    MidSwap,
}

pub struct SdmmcFs<const MAX_OPEN_DIRS: usize, const MAX_OPEN_FILES: usize> {
    state: SdmmcState<MAX_OPEN_DIRS, MAX_OPEN_FILES>,
}

impl<const MAX_OPEN_DIRS: usize, const MAX_OPEN_FILES: usize>
    SdmmcFs<MAX_OPEN_DIRS, MAX_OPEN_FILES>
{
    #[allow(dead_code)]
    pub fn new(sdmmc: H7Sdmmc) -> Self {
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
                    match sdmmc.init(freq) {
                        Ok(_) => {
                            // We just got here because the state is SdmmcState::Sdmmc so this should never fail
                            if let SdmmcState::Sdmmc(sd) =
                                core::mem::replace(&mut self.state, SdmmcState::MidSwap)
                            {
                                self.state = SdmmcState::Controller(Controller::<
                                    _,
                                    _,
                                    MAX_OPEN_DIRS,
                                    MAX_OPEN_FILES,
                                >::new_with_limits(
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
                                log::warn!("SD Card mount failed, retrying...");
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

    pub fn read_file<'p, P: Into<Path<'p>>>(
        &mut self,
        path: P,
        data: &mut [u8],
    ) -> Result<usize, SdmmcFsError> {
        self.find_file(path, FileOpenMode::ReadOnly, |controller, volume, file| {
            controller.read(volume, file, data)
        })?
        .map_err(SdmmcFsError::from)
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

    pub fn ls<'p, P: Into<Path<'p>>>(
        &mut self,
        path: P,
        mut func: impl FnMut(&DirEntry),
    ) -> Result<(), SdmmcFsError> {
        self.find_dir(path, |controller, volume, dir| {
            controller.iterate_dir(volume, dir, &mut func)
        })
        .map_err(SdmmcFsError::from)?
        .map_err(SdmmcFsError::from)
    }

    fn find_dir<'p, R, P: Into<Path<'p>>>(
        &mut self,
        path: P,
        func: impl FnMut(
            &mut Controller<H7SdmmcBlockDev, TimeSource, MAX_OPEN_DIRS, MAX_OPEN_FILES>,
            &Volume,
            &Directory,
        ) -> R,
    ) -> Result<R, SdmmcFsError> {
        match self.state {
            SdmmcState::Controller(ref mut controller) => {
                let path = path.into();
                let mut volume = controller.get_volume(VolumeIdx(0))?;
                let root_dir = controller.open_root_dir(&volume)?;

                let res = find_dir(controller, &mut volume, &root_dir, &mut path.parts(), func);

                controller.close_dir(&volume, root_dir);
                res.ok_or(SdmmcFsError::NotFound)?
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }

    fn find_file<'p, R, P: Into<Path<'p>>>(
        &mut self,
        path: P,
        mode: FileOpenMode,
        func: impl FnMut(
            &mut Controller<H7SdmmcBlockDev, TimeSource, MAX_OPEN_DIRS, MAX_OPEN_FILES>,
            &Volume,
            &mut File,
        ) -> R,
    ) -> Result<R, SdmmcFsError> {
        match self.state {
            SdmmcState::Controller(ref mut controller) => {
                let path = path.into();
                let mut volume = controller.get_volume(VolumeIdx(0))?;
                let root_dir = controller.open_root_dir(&volume)?;

                let res = find_file(
                    controller,
                    &mut volume,
                    &root_dir,
                    mode,
                    &mut path.parts(),
                    func,
                );

                controller.close_dir(&volume, root_dir);
                res.ok_or(SdmmcFsError::NotFound)?
            }
            SdmmcState::Sdmmc(_) => Err(SdmmcFsError::NotMounted),
            SdmmcState::MidSwap => unreachable!(),
        }
    }
}

pub fn print_dir_entry<W: fmt::Write>(writer: &mut W, dir_entry: &DirEntry) -> fmt::Result {
    if !dir_entry.attributes.is_volume() && !dir_entry.attributes.is_hidden() {
        if dir_entry.attributes.is_directory() {
            writeln!(writer, "{:13} {}  <DIR>", dir_entry.name, dir_entry.mtime)?;
        } else {
            writeln!(
                writer,
                "{:13} {}  {} bytes",
                dir_entry.name, dir_entry.mtime, dir_entry.size
            )?;
        }
    }
    Ok(())
}

fn find_dir<
    'p,
    R,
    D: BlockDevice,
    T: embedded_sdmmc::TimeSource,
    const MAX_OPEN_DIRS: usize,
    const MAX_OPEN_FILES: usize,
>(
    controller: &mut Controller<D, T, MAX_OPEN_DIRS, MAX_OPEN_FILES>,
    volume: &mut Volume,
    dir: &Directory,
    path_iter: &mut core::iter::Peekable<impl Iterator<Item = &'p str>>,
    mut func: impl FnMut(&mut Controller<D, T, MAX_OPEN_DIRS, MAX_OPEN_FILES>, &Volume, &Directory) -> R,
) -> Option<Result<R, SdmmcFsError>>
where
    SdmmcFsError: From<embedded_sdmmc::Error<<D as BlockDevice>::Error>>,
{
    if let Some(name) = path_iter.next() {
        match controller.open_dir(volume, dir, name) {
            Ok(new_dir) => {
                log::trace!("OPENED DIR: {}", name);
                let res = find_dir(controller, volume, &new_dir, path_iter, func);
                controller.close_dir(volume, new_dir);
                log::trace!("CLOSED DIR: {}", name);
                res
            }
            Err(e) => Some(Err(SdmmcFsError::from(e))),
        }
    } else {
        Some(Ok(func(controller, volume, dir)))
    }
}

fn find_file<
    'p,
    R,
    D: BlockDevice,
    T: embedded_sdmmc::TimeSource,
    const MAX_OPEN_DIRS: usize,
    const MAX_OPEN_FILES: usize,
>(
    controller: &mut Controller<D, T, MAX_OPEN_DIRS, MAX_OPEN_FILES>,
    volume: &mut Volume,
    dir: &Directory,
    mode: FileOpenMode,
    path_iter: &mut core::iter::Peekable<impl Iterator<Item = &'p str>>,
    mut func: impl FnMut(&mut Controller<D, T, MAX_OPEN_DIRS, MAX_OPEN_FILES>, &Volume, &mut File) -> R,
) -> Option<Result<R, SdmmcFsError>>
where
    SdmmcFsError: From<embedded_sdmmc::Error<<D as BlockDevice>::Error>>,
{
    if let Some(name) = path_iter.next() {
        if path_iter.peek().is_some() {
            match controller.open_dir(volume, dir, name) {
                Ok(new_dir) => {
                    log::trace!("OPENED DIR: {}", name);
                    let res = find_file(controller, volume, &new_dir, mode, path_iter, func);
                    controller.close_dir(volume, new_dir);
                    log::trace!("CLOSED DIR: {}", name);
                    res
                }
                Err(e) => Some(Err(SdmmcFsError::from(e))),
            }
        } else {
            match controller.open_file_in_dir(volume, dir, name, mode) {
                Ok(mut file) => {
                    log::trace!("OPENED FILE: {}", name);
                    let ret = func(controller, volume, &mut file);
                    if let Err(e) = controller.close_file(volume, file) {
                        return Some(Err(SdmmcFsError::from(e)));
                    };
                    log::trace!("CLOSED FILE: {}", name);
                    Some(Ok(ret))
                }
                Err(e) => Some(Err(SdmmcFsError::from(e))),
            }
        }
    } else {
        None
    }
}
