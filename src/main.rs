mod timer;
mod shutdown;

extern crate tokio;
extern crate crossterm;

use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use timer::Timer;
use shutdown::Shutdown;

use crossterm::{execute, cursor, style::Print, terminal};
use crossterm::event::{poll, read, Event, KeyCode};

use tokio::sync;

const UPDATES_PER_SECOND: u64 = 1; // this is only set low because vscode integrated terminal uses tons of cpu otherwise
const SPLITS_Y_OFFSET: u16 = 0;

#[tokio::main]
async fn main() {
    let mut shutdown = Shutdown::new();

    let speed_timer = Arc::new(Mutex::new(Timer::new()));

    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Print(speed_timer.clone().lock().unwrap().get_time_string()),
    ).expect("Failed to initialise the terminal");

    tokio::task::spawn(read_input(speed_timer.clone(), shutdown.trigger_send));
    tokio::task::spawn(tick_timer(speed_timer.clone(), shutdown.signal_recv, shutdown.ack_send));

    // await shutdown trigger from input task
    tokio::select! {
        // _ = tokio::signal::ctrl_c() => (),
        _ = shutdown.trigger_recv => {
            shutdown.signal_send.send(true).expect("Shutdown signal not sent"); // tell all other tasks to shut down
            ()
        },
    };

    // await shutdown acknowledgement from all tasks
    shutdown.ack_recv.recv().await;
    ()
}

async fn read_input(speed_timer: Arc<Mutex<Timer>>, _shutdown_send: sync::oneshot::Sender<()>) {
    loop {
        if poll(Duration::from_millis(500)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let mut speed_timer = speed_timer.lock().unwrap();
            match read() {
                Ok(Event::Key(event)) => {
                    match event.code {
                        KeyCode::Char(' ') => {
                            if speed_timer.is_running() {
                                speed_timer.split();
                                execute!(
                                    stdout(),
                                    cursor::MoveTo(0, speed_timer.get_splits_count() + SPLITS_Y_OFFSET),
                                    Print(speed_timer.get_latest_split())
                                ).expect("Print split failed");
                            }
                        },
                        KeyCode::Char('s') => {
                            if speed_timer.is_running() { speed_timer.stop() } else { speed_timer.start() };
                        },
                        KeyCode::Char('r') => {
                            speed_timer.reset();
                            execute!(
                                stdout(),
                                terminal::Clear(terminal::ClearType::All),
                                cursor::MoveTo(0, 0),
                                Print(speed_timer.get_time_string()),
                            ).expect("Reset timer failed");
                        },
                        KeyCode::Esc => {
                            break; // exiting the loop allows the task to end, which causes `_shutdown_send` to fire
                        },
                        _ => (),
                    };
                    Ok(())
                },
                Ok(Event::Mouse(_event)) => Ok(()),
                Ok(Event::Resize(_width, _height)) => Ok(()),
                Err(_) => Err(()),
            }.expect("Failed to read input");
        }
    }
}

async fn tick_timer(speed_timer: Arc<Mutex<Timer>>, shutdown_recv: sync::watch::Receiver<bool>, _shutdown_send: sync::mpsc::UnboundedSender<()>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));

    loop {
        if *shutdown_recv.borrow() == true {
            break;
        } else {
            let mut speed_timer = speed_timer.lock().unwrap();
            if speed_timer.is_running() {
                execute!(
                    stdout(),
                    cursor::MoveTo(0, 0),
                    Print(speed_timer.get_time_string()),
                ).expect("Failed to print current time");
            }
        }

        interval.tick().await;
    }
}
