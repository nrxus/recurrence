use crate::{tz_date_iterator::TzDateIterator, End};
use chrono::{NaiveDateTime, TimeZone as _};
use chrono_tz::Tz;
use std::time::SystemTime;

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

impl Daily {
    pub fn new(options: Options) -> Self {
        Daily {
            dtstart: from_system_to_naive(options.dtstart.unwrap_or_else(|| SystemTime::now())),
            timezone: options.timezone.unwrap_or_else(local_tz),
            interval: options.interval.unwrap_or(1),
            end: options.end,
        }
    }

    pub fn all(&self) -> impl Iterator<Item = SystemTime> {
        TzDateIterator {
            end: self.end.into(),
            cursor: self.timezone.from_utc_datetime(&self.dtstart),
            interval: chrono::Duration::days(self.interval as i64),
        }
    }

    pub fn after(&self, min: SystemTime) -> impl Iterator<Item = SystemTime> {
        let min = self.timezone.from_utc_datetime(&from_system_to_naive(min));
        let dtstart = self.timezone.from_utc_datetime(&self.dtstart);

        let cursor = if min <= dtstart {
            dtstart
        } else {
            let time = dtstart.time();
            let mut min_date = min.date();
            if time < min.time() {
                min_date = min_date.succ();
            }

            min_date.and_time(time).expect("bug: and_time")
        };

        TzDateIterator {
            end: self.end.into(),
            interval: chrono::Duration::days(self.interval as i64),
            cursor,
        }
    }
}

fn from_system_to_naive(time: SystemTime) -> NaiveDateTime {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).expect("bug");
    NaiveDateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
}

fn local_tz() -> Tz {
    iana_time_zone::get_timezone()
        .expect("bug: could not get tz")
        .parse()
        .expect("bug: local tz could not be parsed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn starts_today() {
        let now = SystemTime::now();
        let daily = super::Daily::new(Options::default());
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
        });

        let first = daily.all().next().unwrap();

        assert_eq!(dtstart, first);
    }

    #[test]
    fn multiple_days() {
        let dtstart = SystemTime::now();
        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });
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
        });
        let count = daily.all().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn until_limit() {
        let daily = super::Daily::new(Options {
            end: End::Until(SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 4 + 5)),
            ..Options::default()
        });

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
        });

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
        });

        let first_day_of_no_dst = daily.all().skip(1).next().unwrap();
        let difference = first_day_of_no_dst.duration_since(last_day_of_dst).unwrap();

        // 25 hours
        assert_eq!(difference, Duration::from_secs(25 * 60 * 60));
    }

    #[test]
    fn after_before_dtstart() {
        let dtstart = SystemTime::now() - Duration::from_secs(1_234_456);

        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = daily
            .after(dtstart - Duration::from_secs(60 * 60 * 40))
            .next()
            .unwrap();

        assert_eq!(dtstart, first);
    }

    #[test]
    fn after_right_after_dtstart() {
        let dtstart = SystemTime::now() - Duration::from_secs(1_234_456);

        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = daily
            .after(dtstart + Duration::from_secs(60))
            .next()
            .unwrap();

        assert_eq!(dtstart + Duration::from_secs(60 * 60 * 24), first);
    }

    #[test]
    fn after_days_after_dtstart() {
        let dtstart = SystemTime::now() - Duration::from_secs(1_234_456);

        let daily = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = daily
            .after(dtstart + Duration::from_secs(60 * 60 * 24 * 5 + 10))
            .next()
            .unwrap();

        assert_eq!(dtstart + Duration::from_secs(60 * 60 * 24 * 6), first);
    }
}
