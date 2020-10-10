pub mod daily;

mod rrule;
mod set;
mod tz_date_iterator;

use std::time::SystemTime;

pub use crate::{daily::Daily, rrule::RRule, set::Set};

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
