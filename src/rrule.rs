use std::time::SystemTime;

pub enum RRule {
    Daily(super::Daily),
}

impl RRule {
    pub fn all(&self) -> impl Iterator<Item = SystemTime> {
        match self {
            RRule::Daily(d) => d.all(),
        }
    }
}

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
        let mut min_heap: std::collections::BinaryHeap<_> = self
            .rules
            .iter()
            .map(RRule::all)
            .filter_map(|mut iter| iter.next().map(|cursor| IterHolder { iter, cursor }))
            .collect();

        std::iter::from_fn(move || {
            min_heap.pop().map(|IterHolder { cursor, mut iter }| {
                if let Some(next) = iter.next() {
                    min_heap.push(IterHolder { cursor: next, iter });
                }
                cursor
            })
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

// created explicitely to be used in a BinaryHeap
// the order is reversed purposefully since BinaryHeap is a max-heap
// but we want a min heap to get the earliest time
impl<I: Iterator<Item = SystemTime>> Ord for IterHolder<I> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cursor.cmp(&other.cursor).reverse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daily::{self, Daily};
    use std::time::Duration;

    #[test]
    fn all() {
        let first_start = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let day_and_a_half_before = first_start - Duration::from_secs(36 * 60 * 60);

        let set = Set::new()
            .rrule(RRule::Daily(
                Daily::new(daily::Options {
                    dtstart: Some(first_start),
                    ..daily::Options::default()
                })
                .unwrap(),
            ))
            .rrule(RRule::Daily(
                Daily::new(daily::Options {
                    dtstart: Some(day_and_a_half_before),
                    ..daily::Options::default()
                })
                .unwrap(),
            ));

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
}
