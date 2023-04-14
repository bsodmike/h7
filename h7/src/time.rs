use {
    crate::utils::interrupt_free,
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike},
    core::cell::RefCell,
    critical_section::Mutex,
    stm32h7xx_hal::rtc::Rtc,
};

pub static RTC: Mutex<RefCell<Option<Rtc>>> = Mutex::new(RefCell::new(None));
pub static BOOT_TIME: Mutex<RefCell<Option<NaiveDateTime>>> = Mutex::new(RefCell::new(None));

const DEFAULT_TIMESTAMP: embedded_sdmmc::Timestamp = embedded_sdmmc::Timestamp {
    year_since_1970: 0,
    zero_indexed_month: 0,
    zero_indexed_day: 0,
    hours: 0,
    minutes: 0,
    seconds: 0,
};

pub struct TimeSource;

impl TimeSource {
    pub fn set_source(rtc: Rtc) {
        interrupt_free(|cs| RTC.borrow(cs).replace(Some(rtc)));
    }

    pub fn set_date(d: NaiveDate) -> Result<(), ()> {
        interrupt_free(|cs| {
            match RTC
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .and_then(|rtc| rtc.time().map(|t| (rtc, t)))
            {
                Some((rtc, t)) => {
                    rtc.set_date_time(NaiveDateTime::new(d, t));
                    Ok(())
                }
                _ => Err(()),
            }
        })
    }

    pub fn set_time(t: NaiveTime) -> Result<(), ()> {
        interrupt_free(|cs| {
            match RTC
                .borrow(cs)
                .borrow_mut()
                .as_mut()
                .and_then(|rtc| rtc.date().map(|d| (rtc, d)))
            {
                Some((rtc, d)) => {
                    rtc.set_date_time(NaiveDateTime::new(d, t));
                    Ok(())
                }
                _ => Err(()),
            }
        })
    }

    pub fn set_date_time(dt: NaiveDateTime) -> Result<(), ()> {
        interrupt_free(|cs| match &mut *RTC.borrow(cs).borrow_mut() {
            Some(rtc) => {
                rtc.set_date_time(dt);
                Ok(())
            }
            None => Err(()),
        })
    }

    pub fn get_date_time() -> Option<NaiveDateTime> {
        interrupt_free(|cs| {
            RTC.borrow(cs)
                .borrow()
                .as_ref()
                .and_then(|dt| dt.date_time())
        })
    }
}

impl embedded_sdmmc::TimeSource for TimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        match { Self::get_date_time() } {
            Some(date_time) => embedded_sdmmc::Timestamp {
                year_since_1970: (date_time.year() - 1970) as u8,
                zero_indexed_month: date_time.month0() as u8,
                zero_indexed_day: date_time.day0() as u8,
                hours: date_time.hour() as u8,
                minutes: date_time.minute() as u8,
                seconds: date_time.second() as u8,
            },
            None => DEFAULT_TIMESTAMP,
        }
    }
}
