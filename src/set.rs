use crate::RRule;
use std::time::SystemTime;

#[derive(Default)]
pub struct Set {
    rules: Vec<RRule>,
}

impl Set {
    pub fn new() -> Self {
        Set::default()
    }

    pub fn rrule(mut self, rule: RRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn all(&self) -> impl Iterator<Item = SystemTime> {
        self.merge_recurrences(RRule::all)
    }

    pub fn after(&self, min: SystemTime) -> impl Iterator<Item = SystemTime> {
        self.merge_recurrences(move |r| r.after(min))
    }

    fn merge_recurrences<F: Iterator<Item = SystemTime>>(
        &self,
        dates: impl Fn(&RRule) -> F,
    ) -> impl Iterator<Item = SystemTime> {
        use std::cmp::Reverse;

        let mut min_heap: std::collections::BinaryHeap<_> = self
            .rules
            .iter()
            .map(dates)
            .filter_map(|mut iter| {
                iter.next()
                    .map(|cursor| Reverse(IterHolder { iter, cursor }))
            })
            .collect();

        std::iter::from_fn(move || {
            while let Some(Reverse(IterHolder { cursor, mut iter })) = min_heap.pop() {
                if let Some(next) = iter.next() {
                    min_heap.push(Reverse(IterHolder { cursor: next, iter }))
                }

                if let Some(Reverse(IterHolder { cursor: next, .. })) = min_heap.peek() {
                    if *next == cursor {
                        continue;
                    }
                }

                return Some(cursor);
            }

            None
        })
    }
}

/// Holds an interator and the latest date that came out of it
pub struct IterHolder<I: Iterator<Item = SystemTime>> {
    cursor: SystemTime,
    iter: I,
}

impl<I: Iterator<Item = SystemTime>> Eq for IterHolder<I> {}

impl<I: Iterator<Item = SystemTime>> PartialEq for IterHolder<I> {
    fn eq(&self, other: &Self) -> bool {
        self.cursor.eq(&other.cursor)
    }
}

impl<I: Iterator<Item = SystemTime>> PartialOrd for IterHolder<I> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: Iterator<Item = SystemTime>> Ord for IterHolder<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cursor.cmp(&other.cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{daily, weekly, Daily, Weekly};
    use std::time::Duration;

    #[test]
    fn all() {
        let first_start = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let day_and_a_half_before = first_start - Duration::from_secs(36 * 60 * 60);

        let set = Set::new()
            .rrule(RRule::Daily(Daily::new(daily::Options {
                dtstart: Some(first_start),
                ..daily::Options::default()
            })))
            .rrule(RRule::Daily(Daily::new(daily::Options {
                dtstart: Some(day_and_a_half_before),
                ..daily::Options::default()
            })));

        let mut all = set.all();
        assert_eq!(all.next().unwrap(), day_and_a_half_before);
        assert_eq!(
            all.next().unwrap(),
            day_and_a_half_before + Duration::from_secs(24 * 60 * 60)
        );
        assert_eq!(all.next().unwrap(), first_start);
        assert_eq!(
            all.next().unwrap(),
            day_and_a_half_before + Duration::from_secs(2 * 24 * 60 * 60)
        );
    }

    #[test]
    fn skips_repeated() {
        let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);

        let set = Set::new()
            .rrule(RRule::Daily(Daily::new(daily::Options {
                dtstart: Some(start),
                ..daily::Options::default()
            })))
            .rrule(RRule::Weekly(Weekly::new(weekly::Options {
                dtstart: Some(start),
                ..weekly::Options::default()
            })));

        let mut all = set.all();
        assert_eq!(all.next().unwrap(), start);
        // the next one is a day after instead of repeating start
        assert_eq!(
            all.next().unwrap(),
            start + Duration::from_secs(24 * 60 * 60)
        );
    }
}
