use chrono::{DateTime, Duration, NaiveDateTime, Offset as _};
use chrono_tz::Tz;
use std::time::SystemTime;

#[derive(Clone, Copy)]
pub enum End {
    Until(NaiveDateTime),
    Count(u32),
    Never,
}

impl From<crate::End> for End {
    fn from(end: crate::End) -> End {
        match end {
            crate::End::Never => End::Never,
            crate::End::Count(count) => End::Count(count),
            crate::End::Until(until) => End::Until(from_system_to_naive(until)),
        }
    }
}

fn from_system_to_naive(time: SystemTime) -> NaiveDateTime {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).expect("bug");
    NaiveDateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
}

/// Timezone Aware Date Iterator
pub struct TzDateIterator {
    pub end: End,
    pub cursor: DateTime<Tz>,
    pub interval: Duration,
}

impl Iterator for TzDateIterator {
    type Item = SystemTime;

    fn next(&mut self) -> Option<SystemTime> {
        match self.end {
            End::Count(0) => return None,
            End::Until(until) if until < self.cursor.naive_utc() => {
                return None
            }
            End::Count(ref mut count) => *count -= 1,
            _ => {}
        }

        let mut next = self.cursor + self.interval;

        if next.offset() != self.cursor.offset() {
            let difference = chrono::Duration::seconds(
                (next.offset().fix().local_minus_utc()
                    - self.cursor.offset().fix().local_minus_utc()) as i64,
            );
            next = next - difference;
        }

        let current = std::mem::replace(&mut self.cursor, next);
        Some(current.into())
    }
}
