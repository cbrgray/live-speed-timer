mod time;

extern crate tokio;

// use async_std::io::{self, WriteExt};
use std::io::Write;
use std::time::Duration;
use time::SplitterTimer;

const UPDATES_PER_SECOND: u64 = 1;

#[tokio::main]
async fn main() {

    let forever = tokio::task::spawn(async move {
        let mut speed_timer = SplitterTimer::new();
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));

        loop {
            speed_timer.update().await;
            print!("\r{}", speed_timer.get_time());
            std::io::stdout().flush().unwrap();
            interval.tick().await;
            // io::stdout().write(&speed_timer.get_time()).await;
            // io::stdout().flush().await;
            // writer.write_all(&speed_timer.get_time()).unwrap();
            // writer.flush().unwrap();
            // print!("\r{}", speed_timer.get_time());
            // std::io::stdout().flush().unwrap();
        }
    });

    forever.await.unwrap();
}
