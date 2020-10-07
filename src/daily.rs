use chrono::{NaiveDateTime, Offset as _, TimeZone as _};
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

            let mut next = cursor + interval;

            if next.offset() != cursor.offset() {
                let difference = chrono::Duration::seconds(
                    (next.offset().fix().local_minus_utc()
                        - cursor.offset().fix().local_minus_utc()) as i64,
                );
                next = next - difference;
            }

            let current = std::mem::replace(&mut cursor, next);
            Some(current.into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use std::time::{Duration, SystemTime};

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
    fn dtstart() {
        let dtstart = SystemTime::now() - Duration::from_secs(1_234_456);

        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        })
        .unwrap();

        let first = daily.all().next().unwrap();

        assert_eq!(dtstart, first);
    }

    #[test]
    fn multiple_days() {
        let dtstart = SystemTime::now();
        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        })
        .unwrap();
        let mut dates = daily.all().skip(1);

        assert_eq!(
            dtstart + Duration::from_secs(60 * 60 * 24),
            dates.next().unwrap(),
        );

        assert_eq!(
            dtstart + Duration::from_secs(60 * 60 * 24 * 2),
            dates.next().unwrap(),
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
            end: End::Until(SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 4 + 5)),
            ..Options::default()
        })
        .unwrap();

        let count = daily.all().count();

        assert_eq!(count, 5);
    }

    #[test]
    fn interval() {
        let dtstart = SystemTime::now();
        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            interval: Some(3),
            ..Options::default()
        })
        .unwrap();

        let three_days_later = daily.all().skip(1).next().unwrap();

        assert_abs_diff_eq!(
            three_days_later.duration_since(dtstart).unwrap().as_secs(),
            60 * 60 * 24 * 3,
        );
    }

    #[test]
    fn dst_changes() {
        let last_day_of_dst =
            SystemTime::from(chrono_tz::US::Eastern.ymd(2019, 11, 2).and_hms(23, 0, 0));

        let daily = super::Daily::new(Options {
            dtstart: Some(last_day_of_dst),
            timezone: Some(chrono_tz::US::Eastern),
            ..Options::default()
        })
        .unwrap();

        let first_day_of_no_dst = daily.all().skip(1).next().unwrap();
        let difference = first_day_of_no_dst.duration_since(last_day_of_dst).unwrap();

        // 25 hours
        assert_eq!(difference, Duration::from_secs(25 * 60 * 60));
    }
}
