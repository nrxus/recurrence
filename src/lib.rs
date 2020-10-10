pub mod daily;
pub mod rrule;

mod tz_date_iterator;

use std::time::SystemTime;

pub use crate::{daily::Daily, rrule::RRule};

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
