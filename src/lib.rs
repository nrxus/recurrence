mod tz_date_iterator;
pub mod daily;

use std::time::SystemTime;

pub use daily::Daily;

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
