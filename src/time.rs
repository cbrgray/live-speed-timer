use std::time::{Duration, Instant};

pub struct SplitterTimer {
    main_timer: Duration,
    start_time: Instant,
    splits: Vec<Duration>,
}

impl SplitterTimer {

    pub fn new() -> SplitterTimer {
        SplitterTimer {
            main_timer: Duration::new(0, 0),
            start_time: Instant::now(),
            splits: vec![],
        }
    }

    /*fn start(&mut self) {
        self.start_time = Instant::now();
    }

    fn stop() {

    }*/

    pub fn time_to_string(duration: Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        return format!("{hours:02}:{minutes:02}:{seconds:02}");
    }

    pub fn time_to_millistring(duration: Duration) -> String {
        let milliseconds = duration.as_millis() % 1000;
        let asdf = SplitterTimer::time_to_string(duration);
        return format!("{asdf}.{milliseconds:03}");
    }

    pub fn get_time(&mut self) -> String {
        return SplitterTimer::time_to_string(self.main_timer);
    }

    pub fn get_latest_split(&self) -> String {
        return SplitterTimer::time_to_millistring(*self.splits.last().unwrap());
    }

    pub fn get_splits_count(&self) -> u16 {
        return self.splits.len().try_into().unwrap();
    }

    pub fn update(&mut self) {
        self.main_timer = self.start_time.elapsed();
    }

    pub fn split(&mut self) {
        self.splits.push(self.start_time.elapsed());
    }
}
