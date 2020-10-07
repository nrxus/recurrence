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
    timezone: hourglass::Timezone,
    dtstart: hourglass::Timespec,
    end: End,
}

#[derive(Default)]
pub struct Options<'a> {
    pub interval: Option<u32>,
    pub dtstart: Option<SystemTime>,
    pub timezone: Option<&'a str>,
    pub end: End,
}

fn timespec(time: SystemTime) -> hourglass::Timespec {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).expect("bug");
    hourglass::Timespec::unix(duration.as_secs() as i64, 0).expect("bug")
}

fn timezone(tz: Option<&str>) -> Result<hourglass::Timezone, Box<dyn Error>> {
    tz.map(hourglass::Timezone::new)
        .unwrap_or_else(hourglass::Timezone::local)
        .map_err(Into::into)
}

impl Daily {
    pub fn new(options: Options) -> Result<Self, Box<dyn Error>> {
        Ok(Daily {
            dtstart: timespec(options.dtstart.unwrap_or_else(|| SystemTime::now())),
            timezone: timezone(options.timezone)?,
            interval: options.interval.unwrap_or(1),
            end: options.end,
        })
    }

    pub fn all(&self) -> impl Iterator<Item = SystemTime> + '_ {
        let mut cursor = self.dtstart.to_datetime(&self.timezone);
        let interval = hourglass::Deltatime::days(i64::from(self.interval));
        let mut end = self.end;

        std::iter::from_fn(move || {
            match end {
                End::Count(0) => return None,
                End::Until(until) if timespec(until) < cursor.to_timespec() => return None,
                End::Count(ref mut count) => *count -= 1,
                _ => {}
            }

            let next = cursor + interval;
            let current = std::mem::replace(&mut cursor, next);
            Some(SystemTime::UNIX_EPOCH + std::time::Duration::new(current.unix() as u64, 0))
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
            60 * 60 * 24 - 1,
        );

        assert_abs_diff_eq!(
            dates.next().unwrap().duration_since(now).unwrap().as_secs(),
            60 * 60 * 24 * 2 - 1,
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
}
