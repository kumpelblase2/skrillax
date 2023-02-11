use crate::{ByteSize, Serialize};
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Datelike, Duration as CDuration, TimeZone, Timelike, Utc};
use std::ops::{Add, Deref};
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct SilkroadTime(DateTime<Utc>);

impl Default for SilkroadTime {
    fn default() -> Self {
        SilkroadTime(Utc::now())
    }
}

impl From<DateTime<Utc>> for SilkroadTime {
    fn from(time: DateTime<Utc>) -> Self {
        Self(time)
    }
}

impl From<Duration> for SilkroadTime {
    fn from(duration: Duration) -> Self {
        let start = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0);
        let new = start.add(CDuration::from_std(duration).unwrap());
        SilkroadTime(new)
    }
}

impl Deref for SilkroadTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for SilkroadTime {
    fn write_to(&self, writer: &mut BytesMut) {
        let data = ((self.year() - 2000) as u32) & 63
            | ((self.month() - 1) & 15) << 6
            | ((self.day() - 1) & 31) << 10
            | (self.hour() & 31) << 15
            | (self.minute() & 63) << 20
            | (self.second() & 63) << 26;
        data.write_to(writer);
    }
}

impl ByteSize for SilkroadTime {
    fn byte_size(&self) -> usize {
        4
    }
}

impl<T: TimeZone> Serialize for DateTime<T> {
    fn write_to(&self, writer: &mut BytesMut) {
        writer.put_u16_le(self.year() as u16);
        writer.put_u16_le(self.month() as u16);
        writer.put_u16_le(self.day() as u16);
        writer.put_u16_le(self.hour() as u16);
        writer.put_u16_le(self.minute() as u16);
        writer.put_u16_le(self.second() as u16);
        writer.put_u32_le(self.timestamp_millis() as u32);
    }
}

impl<T: TimeZone> ByteSize for DateTime<T> {
    fn byte_size(&self) -> usize {
        16
    }
}
