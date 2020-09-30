use chrono::DateTime;

#[derive(Clone, Copy)]
pub enum Frequency {
    Daily,
}

pub struct Recurrence {
    freq: Frequency,
    dtstart: DateTime<chrono::Local>,
}

impl Recurrence {
    pub fn new(freq: Frequency) -> Self {
        Recurrence {
            freq,
            dtstart: chrono::Local::now(),
        }
    }

    pub fn freq(&self) -> Frequency {
        self.freq
    }

    pub fn dtstart(&self) -> DateTime<chrono::Local> {
        self.dtstart
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;

    #[test]
    fn defaults_to_now() {
        let recurrence = Recurrence::new(Frequency::Daily);
        let dtstart = recurrence.dtstart();
        assert_abs_diff_eq!(dtstart.timestamp(), chrono::Local::now().timestamp());
    }
}
