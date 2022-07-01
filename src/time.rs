use std::time::{Duration, Instant};

pub struct SplitterTimer {
    current_total: Duration,
    current_segment: Duration,
    start_time: Instant,
    splits: Vec<Duration>,
    pub is_running: bool,
}

impl SplitterTimer {

    pub fn new() -> SplitterTimer {
        SplitterTimer {
            current_total: Duration::new(0, 0),
            current_segment: Duration::new(0, 0),
            start_time: Instant::now(),
            splits: vec![],
            is_running: false,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Instant::now();
        self.is_running = true;
    }

    pub fn stop(&mut self) {
        self.current_total += self.current_segment;
        self.is_running = false;
    }

    pub fn reset(&mut self) {
        self.is_running = false;
        self.current_total = Duration::new(0, 0);
        self.current_segment = Duration::new(0, 0);
        self.splits = vec![];
    }

    pub fn time_to_string(duration: Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = (duration.as_secs() / 60) / 60;
        return format!("{hours:02}:{minutes:02}:{seconds:02}");
    }

    fn time_to_millistring(duration: Duration) -> String {
        let milliseconds = duration.as_millis() % 1000;
        let basic_time = SplitterTimer::time_to_string(duration);
        return format!("{basic_time}.{milliseconds:03}");
    }

    pub fn get_time(&self) -> String {
        return SplitterTimer::time_to_string(self.current_total + self.current_segment);
    }

    pub fn get_latest_split(&self) -> String {
        return SplitterTimer::time_to_millistring(*self.splits.last().unwrap());
    }

    pub fn get_splits_count(&self) -> u16 {
        return self.splits.len().try_into().unwrap();
    }

    pub fn update(&mut self) {
        self.current_segment = self.start_time.elapsed();
    }

    pub fn split(&mut self) {
        self.update();
        self.splits.push(self.current_total + self.current_segment);
    }
}
