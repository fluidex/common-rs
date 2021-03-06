use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn system_time_to_timestamp(t: SystemTime) -> f64 {
    t.duration_since(UNIX_EPOCH).unwrap().as_micros() as f64 / 1_000_000_f64
}

pub fn timestamp_to_system_time(t: f64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs_f64(t)
}

pub fn current_system_time() -> SystemTime {
    SystemTime::now()
}

pub fn current_timestamp() -> f64 {
    system_time_to_timestamp(current_system_time())
}

pub fn current_naive_time() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

pub struct FTimestamp(pub f64);

impl From<FTimestamp> for f64 {
    fn from(f: FTimestamp) -> f64 {
        f.0
    }
}

impl From<&f64> for FTimestamp {
    fn from(f: &f64) -> FTimestamp {
        FTimestamp(*f)
    }
}

impl From<FTimestamp> for NaiveDateTime {
    fn from(f: FTimestamp) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(f.0 as i64, ((f.0 - f.0 as i64 as f64) * 1e9) as u32)
    }
}

impl From<&NaiveDateTime> for FTimestamp {
    fn from(f: &NaiveDateTime) -> FTimestamp {
        FTimestamp(f.timestamp_nanos() as f64 / 1e9)
    }
}

impl From<&DateTime<Utc>> for FTimestamp {
    fn from(f: &DateTime<Utc>) -> FTimestamp {
        FTimestamp(f.timestamp() as f64)
    }
}

impl From<FTimestamp> for DateTime<Utc> {
    fn from(f: FTimestamp) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(f.into(), Utc)
    }
}

impl FTimestamp {
    pub fn as_seconds(&self) -> i64 {
        self.0.floor() as i64
    }

    pub fn as_milliseconds(&self) -> i64 {
        (self.0 * 1e3).floor() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_convert_to_seconds() {
        let timestamp: FTimestamp = (&NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 40)).into();
        assert_eq!(timestamp.as_seconds(), 40);
    }

    #[test]
    fn test_convert_to_milliseconds() {
        let timestamp: FTimestamp =
            (&NaiveDate::from_ymd(1970, 1, 1).and_hms_milli(0, 0, 40, 50)).into();
        assert_eq!(timestamp.as_milliseconds(), 40050);
    }
}
