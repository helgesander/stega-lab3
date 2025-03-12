use std::time::{SystemTime, UNIX_EPOCH};

pub struct ParkMiller {
    state: i64,
}

impl ParkMiller {
    pub(crate) fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        ParkMiller { state: now % 100 }
    }

    fn next(&mut self) -> i16 {
        self.state = (self.state * 16807) % 2147483647;
        let value = (self.state as f64) / 2147483647.0 + 0.000000000233;

        if value > 0.5 {
            1
        } else {
            -1
        }
    }

    pub fn generate_prs(&mut self, num: usize) -> Vec<i16> {
        let mut prs: Vec<i16> = Vec::new();
        for _ in 0..num {
            prs.push(self.next());
        }
        prs
    }
}