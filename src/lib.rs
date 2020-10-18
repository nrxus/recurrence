pub mod daily;
pub mod weekly;

mod rrule;
mod set;
mod tz_date_iterator;

use std::time::SystemTime;

pub use crate::{daily::Daily, rrule::RRule, set::Set, weekly::Weekly};

#[derive(Clone, Copy)]
pub enum End {
    Until(SystemTime),
    Count(usize),
    Never,
}

impl Default for End {
    fn default() -> Self {
        End::Never
    }
}

#[cfg(test)]
pub mod test_helpers {
    use std::time::{SystemTime, Duration};

    pub const ONE_MINUTE: Duration = Duration::from_secs(60);
    pub const ONE_HOUR: Duration = Duration::from_secs(60 * ONE_MINUTE.as_secs());
    pub const ONE_DAY: Duration = Duration::from_secs(24 * ONE_HOUR.as_secs());
    pub const ONE_WEEK: Duration = Duration::from_secs(7 * ONE_DAY.as_secs());

    pub fn july_first() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1593576285)
    }
}
