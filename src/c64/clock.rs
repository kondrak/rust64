// timing clock structure
extern crate time;

pub struct Clock {
    curr_time: f64,
    last_time: f64,
    clock_period: f64,
}

impl Clock {
    pub fn new(freq: f64) -> Clock {
        let mut clock = Clock {
            curr_time: 0.0,
            last_time: 0.0,
            clock_period: 1.0 / freq,
        };

        clock.last_time = time::precise_time_s();
        clock
    }

    pub fn tick(&mut self) -> bool {
        self.curr_time = time::precise_time_s();

        if self.curr_time - self.last_time >= self.clock_period {
            self.last_time = self.curr_time;
            return true
        }

        false
    }
}
