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
        let start = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_convert_time() {
        let one_year = 60 * 60 * 24 * 366u64;
        let one_day = 60 * 60 * 24u64;

        let time_now = Duration::from_secs(one_year + one_day + 35);
        let sro_time = SilkroadTime::from(time_now);
        let mut bytes = BytesMut::new();
        sro_time.write_to(&mut bytes);
        let written_bytes = bytes.freeze();

        assert_eq!(written_bytes.len(), 4);

        let lowest = written_bytes[0];
        assert_eq!(lowest, 1); // The lowest 6 bits contain the year since year 2000, thus should be 1

        let second = written_bytes[1];
        assert_eq!(second >> 2, 1); // We need to shift by two to get the day part from the second byte

        let highest = written_bytes[3];
        assert_eq!(highest >> 2, 35);
    }
}
