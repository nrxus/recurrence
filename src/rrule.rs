use std::time::SystemTime;

pub enum RRule {
    Daily(super::Daily),
    Weekly(super::Weekly),
}

impl RRule {
    pub fn all(&self) -> impl Iterator<Item = SystemTime> {
        match self {
            RRule::Daily(d) => Box::new(d.all()) as Box<dyn Iterator<Item = _>>,
            RRule::Weekly(w) => Box::new(w.all()),
        }
    }

    pub fn after(&self, min: SystemTime) -> impl Iterator<Item = SystemTime> {
        match self {
            RRule::Daily(d) => Box::new(d.after(min)) as Box<dyn Iterator<Item = _>>,
            RRule::Weekly(w) => Box::new(w.after(min)),
        }
    }
}
