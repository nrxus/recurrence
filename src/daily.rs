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
        let mut end = self.end;

        let cursor = if min <= dtstart {
            dtstart
        } else {
            let time = dtstart.time();
            let start_date = dtstart.date();
            let mut date = min.date();

            if time < min.time() {
                date = date.succ();
            }

            if let End::Count(ref mut c) = end {
                *c -= (date - start_date).num_days() as usize;
            }

            date.and_time(time).expect("bug: and_time")
        };

        TzDateIterator {
            end: end.into(),
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
    use crate::test_helpers::*;
    use approx::*;
    use std::time::SystemTime;

    #[test]
    fn starts_today() {
        let now = SystemTime::now();
        let dates = super::Daily::new(Options::default());
        let mut dates = dates.all();

        assert_abs_diff_eq!(
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            dates
                .next()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    #[test]
    fn dtstart() {
        let dtstart = july_first();

        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates.all().nth(0).unwrap();

        assert_eq!(dtstart, first);
    }

    #[test]
    fn multiple_days() {
        let dtstart = july_first();
        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });
        let mut dates = dates.all().skip(1);

        assert_eq!(dtstart + ONE_DAY, dates.next().unwrap());
        assert_eq!(dtstart + 2 * ONE_DAY, dates.next().unwrap());
    }

    #[test]
    fn count_limit() {
        let dates = super::Daily::new(Options {
            end: End::Count(2),
            ..Options::default()
        });
        let count = dates.all().count();
        assert_eq!(2, count);
    }

    #[test]
    fn until_limit() {
        let dates = super::Daily::new(Options {
            end: End::Until(SystemTime::now() + 5 * ONE_DAY + ONE_MINUTE),
            ..Options::default()
        });

        let count = dates.all().count();

        assert_eq!(6, count);
    }

    #[test]
    fn interval() {
        let dtstart = july_first();
        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            interval: Some(3),
            ..Options::default()
        });

        let three_days_later = dates.all().nth(1).unwrap();
        assert_eq!(dtstart + 3 * ONE_DAY, three_days_later);
    }

    #[test]
    fn dst_changes() {
        let last_day_of_dst =
            SystemTime::from(chrono_tz::US::Eastern.ymd(2019, 11, 2).and_hms(23, 0, 0));

        let dates = super::Daily::new(Options {
            dtstart: Some(last_day_of_dst),
            timezone: Some(chrono_tz::US::Eastern),
            ..Options::default()
        });

        let first_day_of_no_dst = dates.all().nth(1).unwrap();
        assert_eq!(last_day_of_dst + ONE_DAY + ONE_HOUR, first_day_of_no_dst);
    }

    #[test]
    fn after_before_dtstart() {
        let dtstart = july_first();

        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates.after(dtstart - 40 * ONE_HOUR).nth(0).unwrap();
        assert_eq!(dtstart, first);
    }

    #[test]
    fn after_right_after_dtstart() {
        let dtstart = july_first();

        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates.after(dtstart + ONE_MINUTE).next().unwrap();
        assert_eq!(dtstart + ONE_DAY, first);
    }

    #[test]
    fn after_days_after_dtstart() {
        let dtstart = july_first();

        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates
            .after(dtstart + 5 * ONE_DAY + ONE_MINUTE)
            .nth(0)
            .unwrap();

        assert_eq!(dtstart + 6 * ONE_DAY, first);
    }

    #[test]
    fn after_with_count() {
        let dtstart = july_first();

        let dates = super::Daily::new(Options {
            dtstart: Some(dtstart),
            end: End::Count(5),
            ..Options::default()
        });

        // 5 count as expected
        assert_eq!(5, dates.all().count());

        // but only 1 if we are looking at starting 4 days later
        assert_eq!(1, dates.after(dtstart + 4 * ONE_DAY).count());
    }
}
