use chrono::DateTime;

#[derive(Clone, Copy)]
pub enum Frequency {
    Daily,
}

pub struct Recurrence {
    freq: Frequency,
    interval: u32,
}

impl Recurrence {
    pub fn new(freq: Frequency) -> Self {
        Recurrence {
            freq,
            interval: 1,
        }
    }

    pub fn freq(self, freq: Frequency) -> Self {
        Recurrence { freq, ..self }
    }

    pub fn interval(self, interval: u32) -> Self {
        Recurrence { interval, ..self }
    }

    pub fn all(&self) -> impl Iterator<Item = DateTime<chrono::Utc>> {
        let mut cursor = chrono::Utc::now();
        let interval = match self.freq {
            Frequency::Daily => chrono::Duration::days(self.interval.into())
        };

        std::iter::from_fn(move || {
            let next = cursor + interval;
            Some(std::mem::replace(&mut cursor, next))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    #[test]
    fn starts_today() {
        let now = chrono::Utc::now();
        let recurrence = Recurrence::new(Frequency::Daily);
        let mut dates = recurrence.all();
        assert_abs_diff_eq!(dates.next().unwrap().timestamp(), now.timestamp());
    }

    #[test]
    fn daily() {
        let now = chrono::Utc::now();
        let recurrence = Recurrence::new(Frequency::Daily);
        let mut dates = recurrence.all().skip(1);

        assert_abs_diff_eq!(
            dates.next().unwrap().timestamp(),
            (now + chrono::Duration::days(1)).timestamp()
        );
        assert_abs_diff_eq!(
            dates.next().unwrap().timestamp(),
            (now + chrono::Duration::days(2)).timestamp()
        );
    }

    #[test]
    fn multi_daily() {
        let now = chrono::Utc::now();
        let recurrence = Recurrence::new(Frequency::Daily).interval(4);
        let mut dates = recurrence.all().skip(1);

        assert_abs_diff_eq!(
            dates.next().unwrap().timestamp(),
            (now + chrono::Duration::days(4)).timestamp()
        );
        assert_abs_diff_eq!(
            dates.next().unwrap().timestamp(),
            (now + chrono::Duration::days(8)).timestamp()
        );
    }
}
