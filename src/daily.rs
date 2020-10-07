use chrono::{NaiveDateTime, TimeZone as _};
use chrono_tz::Tz;
use std::{error::Error, time::SystemTime};

#[derive(Clone, Copy)]
pub enum End {
    Until(SystemTime),
    Count(u32),
    Never,
}

impl Default for End {
    fn default() -> Self {
        End::Never
    }
}

pub struct Daily {
    interval: u32,
    timezone: Tz,
    dtstart: NaiveDateTime,
    end: End,
}

#[derive(Default)]
pub struct Options {
    pub interval: Option<u32>,
    pub dtstart: Option<SystemTime>,
    pub timezone: Option<Tz>,
    pub end: End,
}

fn timespec(time: SystemTime) -> NaiveDateTime {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).expect("bug");
    NaiveDateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
}

fn local_tz() -> Tz {
    iana_time_zone::get_timezone()
        .expect("bug: could not get tz")
        .parse()
        .expect("bug: local tz could not be parsed")
}

impl Daily {
    pub fn new(options: Options) -> Result<Self, Box<dyn Error>> {
        Ok(Daily {
            dtstart: timespec(options.dtstart.unwrap_or_else(|| SystemTime::now())),
            timezone: options.timezone.unwrap_or_else(local_tz),
            interval: options.interval.unwrap_or(1),
            end: options.end,
        })
    }

    pub fn all(&self) -> impl Iterator<Item = SystemTime> + '_ {
        let mut cursor = self.timezone.from_utc_datetime(&self.dtstart);
        let interval = chrono::Duration::days(self.interval as i64);
        let mut end = self.end;

        std::iter::from_fn(move || {
            match end {
                End::Count(0) => return None,
                End::Until(until) if timespec(until) < cursor.naive_utc() => return None,
                End::Count(ref mut count) => *count -= 1,
                _ => {}
            }

            let next = cursor + interval;
            let current = std::mem::replace(&mut cursor, next);
            Some(current.into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use std::time::SystemTime;

    #[test]
    fn starts_today() {
        let now = SystemTime::now();
        let daily = super::Daily::new(Options::default()).unwrap();
        let mut dates = daily.all();

        assert_abs_diff_eq!(
            dates
                .next()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    #[test]
    fn multiple_days() {
        let now = SystemTime::now();
        let daily = super::Daily::new(Options::default()).unwrap();
        let mut dates = daily.all().skip(1);

        assert_abs_diff_eq!(
            dates.next().unwrap().duration_since(now).unwrap().as_secs(),
            60 * 60 * 24,
        );

        assert_abs_diff_eq!(
            dates.next().unwrap().duration_since(now).unwrap().as_secs(),
            60 * 60 * 24 * 2,
        );
    }

    #[test]
    fn count_limit() {
        let daily = super::Daily::new(Options {
            end: End::Count(2),
            ..Options::default()
        })
        .unwrap();
        let count = daily.all().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn until_limit() {
        let daily = super::Daily::new(Options {
            end: End::Until(
                SystemTime::now() + std::time::Duration::from_secs(60 * 60 * 24 * 4 + 5),
            ),
            ..Options::default()
        })
        .unwrap();

        let count = daily.all().count();

        assert_eq!(count, 5);
    }

    #[test]
    fn interval() {
        let now = SystemTime::now();
        let daily = super::Daily::new(Options {
            interval: Some(3),
            ..Options::default()
        })
        .unwrap();

        let three_days_later = daily.all().skip(1).next().unwrap();

        assert_abs_diff_eq!(
            three_days_later.duration_since(now).unwrap().as_secs(),
            60 * 60 * 24 * 3,
        );
    }
}
