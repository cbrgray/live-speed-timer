mod timer;
mod shutdown;
mod config;

extern crate tokio;
extern crate crossterm;

use std::collections::HashMap;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use timer::Timer;
use shutdown::Shutdown;
use config::Config;

use crossterm::{execute, cursor, style::Print, terminal};
use crossterm::event::{poll, read, Event, KeyCode};

use tokio::sync;

const WINDOW_TITLE: &str = "LiveSpeedTimer";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const CFG_FILENAME: &str = "cfg.yaml";
const SPLITS_Y_OFFSET: u16 = 0;

#[tokio::main]
async fn main() {
    let cfg = Config::load_config(CFG_FILENAME);
    
    let mut shutdown = Shutdown::new();

    let timer = Arc::new(Mutex::new(Timer::new(cfg)));

    init_term(&timer.clone().lock().unwrap().get_time_string());

    tokio::task::spawn(read_input(timer.clone(), cfg, shutdown.trigger_send));
    tokio::task::spawn(tick_timer(timer.clone(), cfg, shutdown.signal_recv, shutdown.ack_send));

    // await shutdown trigger from input task
    tokio::select! {
        // _ = tokio::signal::ctrl_c() => (),
        _ = shutdown.trigger_recv => {
            shutdown.signal_send.send(true).expect("Shutdown signal not sent"); // tell all other tasks to shut down
        },
    };

    restore_term();

    // await shutdown acknowledgement from all tasks
    shutdown.ack_recv.recv().await;
}

fn init_term(initial_text: &str) {
    terminal::enable_raw_mode().expect("Failed to enable crossterm raw mode");
    execute!(
        stdout(),
        terminal::SetTitle(format!("{} {}", WINDOW_TITLE, VERSION)),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Print(initial_text),
    ).expect("Failed to initialise the terminal");
}

fn restore_term() {
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Show,
        cursor::MoveTo(0, 0),
    ).expect("Failed to restore the terminal on shutdown");
    terminal::disable_raw_mode().expect("Failed to disable crossterm raw mode");
}

async fn read_input(
    timer: Arc<Mutex<Timer>>,
    cfg: Config,
    _shutdown_send: sync::oneshot::Sender<()>,
) {
    let mut bindings = HashMap::new();
    add_input_bindings(&mut bindings, cfg);

    loop {
        if poll(Duration::from_secs(1)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let mut timer = timer.lock().unwrap();
            match read() {
                Ok(Event::Key(event)) => {
                    try_run_input(event.code, &bindings, &mut timer);
                    if event.code == cfg.get_key_quit() {
                        break; // exiting the loop allows the task to end, which triggers `_shutdown_send`
                    }
                },
                Ok(Event::Mouse(_event)) => (),
                Ok(Event::Resize(_width, _height)) => (),
                Err(_) => (),
            };
        }
    }
}

async fn tick_timer(
    timer: Arc<Mutex<Timer>>,
    cfg: Config,
    shutdown_recv: sync::watch::Receiver<bool>,
    _shutdown_send: sync::mpsc::UnboundedSender<()>,
) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / cfg.get_ups()));

    loop {
        if *shutdown_recv.borrow() {
            break;
        } else {
            let mut timer = timer.lock().unwrap();
            if timer.is_running() {
                execute!(
                    stdout(),
                    cursor::MoveTo(0, 0),
                    Print(timer.get_time_string()),
                ).expect("Failed to print current time");
            }
        }

        interval.tick().await;
    }
}

// User input functions

fn stopstart(timer: &mut Timer) {
    if timer.is_running() { timer.stop() } else { timer.start() };
}

fn reset(timer: &mut Timer) {
    timer.reset();
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        Print(timer.get_time_string()),
    ).expect("Reset timer failed");
}

fn split(timer: &mut Timer) {
    if timer.is_running() {
        timer.split();
        execute!(
            stdout(),
            cursor::MoveTo(0, timer.get_splits_count() + SPLITS_Y_OFFSET),
            Print(timer.get_latest_split()),
        ).expect("Print split failed");
    }
}

fn add_input_bindings<'a: 'b, 'b>(
    bindings: &'b mut HashMap<KeyCode, &'a dyn Fn(&mut Timer)>,
    cfg: Config,
) {
    bindings.insert(cfg.get_key_stopstart(), &stopstart);
    bindings.insert(cfg.get_key_reset(), &reset);
    bindings.insert(cfg.get_key_split(), &split);
}

fn try_run_input(
    data: KeyCode,
    bindings: &HashMap<KeyCode,
    &dyn Fn(&mut Timer)>,
    timer: &mut Timer,
) {
    if let Some(x) = bindings.get(&data) {
        x(timer)
    }
}
