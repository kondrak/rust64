extern crate time;

// clock frequency in Hz
static CLOCK_FREQ: f64 = 1.0; //1.0 / 985248.0;

pub struct Clock
{
    curr_time: f64,
    last_time: f64,
}

impl Clock
{
    pub fn new() -> Clock
    {
        let mut clock = Clock
        {
            curr_time: 0.0,
            last_time: 0.0,
        };

        clock.last_time = time::precise_time_s();
        clock
    }

    pub fn tick(&mut self) -> bool
    {
        self.curr_time = time::precise_time_s();

        if self.curr_time - self.last_time > CLOCK_FREQ
        {
            self.last_time = self.curr_time;
            return true
        }

        false
    }
}
