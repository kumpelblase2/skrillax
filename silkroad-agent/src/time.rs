use chrono::Duration as CDuration;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::ops::Add;
use std::time::Duration;

pub trait AsSilkroadTime {
    fn as_silkroad_time(&self) -> u32;
}

impl<T> AsSilkroadTime for DateTime<T>
where
    T: TimeZone,
{
    fn as_silkroad_time(&self) -> u32 {
        ((self.year() - 2000) as u32) & 63
            | (self.month() - 1 & 15) << 6
            | (self.day() - 1 & 31) << 10
            | (self.hour() & 31) << 15
            | (self.minute() & 63) << 20
            | (self.second() & 63) << 26
    }
}

impl AsSilkroadTime for Duration {
    fn as_silkroad_time(&self) -> u32 {
        let start = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
        let new = start.add(CDuration::from_std(self.clone()).unwrap());
        new.as_silkroad_time()
    }
}
