mod time;

extern crate tokio;
extern crate crossterm;

use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::SplitterTimer;

use crossterm::{execute, cursor, style::Print, terminal};
use crossterm::event::{poll, read, Event, KeyCode};

use tokio::sync;

const UPDATES_PER_SECOND: u64 = 1; // this is only set low because vscode integrated terminal uses tons of cpu otherwise
const SPLITS_Y_OFFSET: u16 = 0;

#[tokio::main]
async fn main() {
    let (shutdown_trigger_send, shutdown_trigger_recv) = sync::oneshot::channel::<()>();
    let (shutdown_signal_send, shutdown_signal_recv) = sync::watch::channel(false);
    let (shutdown_ack_send, mut shutdown_ack_recv) = sync::mpsc::unbounded_channel::<()>();

    let speed_timer = Arc::new(Mutex::new(SplitterTimer::new()));
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
    ).expect("Failed to initialise the terminal");

    tokio::task::spawn(read_input(speed_timer.clone(), shutdown_trigger_send));
    tokio::task::spawn(tick_timer(speed_timer.clone(), shutdown_signal_recv, shutdown_ack_send));

    // await shutdown trigger from input task
    tokio::select! {
        // _ = tokio::signal::ctrl_c() => (),
        _ = shutdown_trigger_recv => {
            shutdown_signal_send.send(true).expect("Shutdown signal not sent"); // tell all other tasks to shut down
            ()
        },
    };

    // await shutdown acknowledgement from all tasks
    shutdown_ack_recv.recv().await;
    ()
}

async fn read_input(speed_timer: Arc<Mutex<SplitterTimer>>, _shutdown_send: sync::oneshot::Sender<()>) {
    loop {
        if poll(Duration::from_millis(500)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let mut speed_timer = speed_timer.lock().unwrap();
            match read() {
                Ok(Event::Key(event)) => {
                    match event.code {
                        KeyCode::Char(' ') => {
                            speed_timer.split();
                            execute!(
                                stdout(),
                                cursor::MoveTo(0, speed_timer.get_splits_count() + SPLITS_Y_OFFSET),
                                Print(speed_timer.get_latest_split())
                            ).expect("Print split failed");
                        },
                        KeyCode::Esc => {
                            // speed_timer.stop();
                            break; // exiting the loop allows the task to end, which causes `_shutdown_send` to fire
                        },
                        _ => todo!(),
                    };
                    Ok::<(), ()>(())
                },
                Ok(Event::Mouse(_event)) => todo!(),
                Ok(Event::Resize(_width, _height)) => todo!(),
                Err(_) => todo!(),
            }.expect("Failed to read input");
        }
    }
}

async fn tick_timer(speed_timer: Arc<Mutex<SplitterTimer>>, shutdown_recv: sync::watch::Receiver<bool>, _shutdown_send: sync::mpsc::UnboundedSender<()>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));

    loop {
        if *shutdown_recv.borrow() == true { break; };
        {
            let mut speed_timer = speed_timer.lock().unwrap();
            speed_timer.update();
            execute!(
                stdout(),
                cursor::MoveTo(0, 0),
                Print(speed_timer.get_time()),
            ).expect("Failed to print current time");
        }

        interval.tick().await;
    }
}
