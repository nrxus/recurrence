use crate::{tz_date_iterator::TzDateIterator, End};
use chrono::{Datelike as _, Duration, NaiveDateTime, TimeZone as _};
use chrono_tz::Tz;
use std::time::SystemTime;

pub struct Weekly {
    interval: u32,
    timezone: Tz,
    dtstart: NaiveDateTime,
    end: End,
}

#[derive(Default)]
pub struct Options {
    pub interval: Option<u32>,
    pub timezone: Option<Tz>,
    pub dtstart: Option<SystemTime>,
    pub end: End,
}

impl Weekly {
    pub fn new(options: Options) -> Self {
        Weekly {
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
            interval: chrono::Duration::weeks(self.interval as i64),
        }
    }

    pub fn after(&self, min: SystemTime) -> impl Iterator<Item = SystemTime> {
        let min = self.timezone.from_utc_datetime(&from_system_to_naive(min));
        let dtstart = self.timezone.from_utc_datetime(&self.dtstart);
        let mut end = self.end;

        let cursor = if min <= dtstart {
            dtstart
        } else {
            const DAYS_IN_WEEK: u32 = 7;
            let time = dtstart.time();
            let start_date = dtstart.date();

            let date = {
                let date = min.date();
                let mut difference = (start_date.weekday().number_from_monday() + DAYS_IN_WEEK
                    - date.weekday().number_from_monday())
                    % DAYS_IN_WEEK;

                if difference == 0 && time < min.time() {
                    difference = 7;
                }

                date + Duration::days(difference as i64)
            };

            if let End::Count(ref mut c) = end {
                *c = c.saturating_sub((date - start_date).num_weeks() as usize);
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
    use crate::test_helpers::*;

    use super::*;
    use approx::*;
    use std::time::SystemTime;

    #[test]
    fn starts_today() {
        let now = SystemTime::now();
        let daily = super::Weekly::new(Options::default());
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
        let dtstart = july_first();

        let daily = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = daily.all().nth(0).unwrap();

        assert_eq!(dtstart, first);
    }

    #[test]
    fn multiple_weeks() {
        let dtstart = july_first();
        let daily = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });
        let mut dates = daily.all().skip(1);

        assert_eq!(dtstart + ONE_WEEK, dates.next().unwrap());
        assert_eq!(dtstart + 2 * ONE_WEEK, dates.next().unwrap());
    }

    #[test]
    fn count_limit() {
        let dates = super::Weekly::new(Options {
            end: End::Count(2),
            ..Options::default()
        });
        let count = dates.all().count();
        assert_eq!(2, count);
    }

    #[test]
    fn until_limit() {
        let dates = super::Weekly::new(Options {
            end: End::Until(SystemTime::now() + 3 * ONE_WEEK + ONE_DAY),
            ..Options::default()
        });

        let count = dates.all().count();

        assert_eq!(4, count);
    }

    #[test]
    fn interval() {
        let dtstart = july_first();
        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            interval: Some(4),
            ..Options::default()
        });

        let four_weeks_later = dates.all().nth(1).unwrap();
        assert_eq!(dtstart + 4 * ONE_WEEK, four_weeks_later);
    }

    #[test]
    fn dst_changes() {
        let last_day_of_dst =
            SystemTime::from(chrono_tz::US::Eastern.ymd(2019, 11, 2).and_hms(23, 0, 0));

        let dates = super::Weekly::new(Options {
            dtstart: Some(last_day_of_dst),
            timezone: Some(chrono_tz::US::Eastern),
            ..Options::default()
        });

        let first_week_of_dst = dates.all().nth(1).unwrap();
        assert_eq!(last_day_of_dst + ONE_WEEK + ONE_HOUR, first_week_of_dst);
    }

    #[test]
    fn after_before_dtstart() {
        let dtstart = july_first();

        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates.after(dtstart - 40 * ONE_HOUR).nth(0).unwrap();
        assert_eq!(dtstart, first);
    }

    #[test]
    fn after_right_after_dtstart() {
        let dtstart = july_first();

        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates.after(dtstart + ONE_MINUTE).nth(0).unwrap();
        assert_eq!(dtstart + ONE_WEEK, first);
    }

    #[test]
    fn after_weeks_after_dtstart() {
        let dtstart = july_first();

        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            ..Options::default()
        });

        let first = dates
            .after(dtstart + 2 * ONE_WEEK + ONE_DAY)
            .nth(0)
            .unwrap();

        assert_eq!(dtstart + 3 * ONE_WEEK, first);
    }

    #[test]
    fn after_with_count() {
        let dtstart = july_first();

        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            end: End::Count(4),
            ..Options::default()
        });

        // 4 count as expected
        assert_eq!(dates.all().count(), 4);

        // but only 2 if we are looking at starting 12 days later
        assert_eq!(dates.after(dtstart + 12 * ONE_DAY).count(), 2);
    }

    #[test]
    fn after_too_late() {
        let dtstart = july_first();

        let dates = super::Weekly::new(Options {
            dtstart: Some(dtstart),
            end: End::Count(1),
            ..Options::default()
        });

        assert_eq!(dates.after(dtstart + 12 * ONE_DAY).count(), 0);
    }
}
