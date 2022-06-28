mod time;

extern crate tokio;
extern crate crossterm;

use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::SplitterTimer;

use crossterm::{execute, ExecutableCommand, cursor, style::Print, terminal};
use crossterm::event::{poll, read, Event, KeyCode};

const UPDATES_PER_SECOND: u64 = 1;
const SPLITS_Y_OFFSET: u16 = 0;

#[tokio::main]
async fn main() {
    let speed_timer = Arc::new(Mutex::new(SplitterTimer::new()));
    stdout().execute(terminal::Clear(terminal::ClearType::All)).expect("Failed to clear terminal");

    let a = speed_timer.clone();
    let b = speed_timer.clone();

    tokio::task::spawn(async move {
        loop {
            if poll(Duration::from_millis(500)).unwrap() {
                // It's guaranteed that the `read()` won't block when the `poll()`
                // function returns `true`
                match read() {
                    Ok(Event::Key(event)) => {
                        if event.code == KeyCode::Char(' ') {
                            let mut speed_timer = a.lock().unwrap();
                            speed_timer.split();
                            execute!(stdout(), cursor::MoveTo(0, speed_timer.get_splits_count() + SPLITS_Y_OFFSET), Print(speed_timer.get_latest_split()))
                        } else {
                            Result::Ok(())
                        }
                    },
                    Ok(Event::Mouse(_event)) => todo!(),
                    Ok(Event::Resize(_width, _height)) => todo!(),
                    Err(_) => todo!(),
                }.expect("Failed to read input");
            }
        }
    });

    let forever = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));

        loop {
            {
                let mut speed_timer = b.lock().unwrap();
                speed_timer.update();
                execute!(
                    stdout(),
                    cursor::MoveTo(0, 0),
                    Print(speed_timer.get_time()),
                ).expect("Failed to print current time");
            }

            interval.tick().await;
        }
    });

    forever.await.unwrap();
}
