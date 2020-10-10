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
