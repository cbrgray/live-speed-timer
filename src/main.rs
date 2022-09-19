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

    let speed_timer = Arc::new(Mutex::new(Timer::new()));

    // init term
    terminal::enable_raw_mode().unwrap();
    execute!(
        stdout(),
        terminal::SetTitle(format!("{} {}", WINDOW_TITLE, VERSION)),
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Print(speed_timer.clone().lock().unwrap().get_time_string()),
    ).expect("Failed to initialise the terminal");

    tokio::task::spawn(read_input(speed_timer.clone(), cfg, shutdown.trigger_send));
    tokio::task::spawn(tick_timer(speed_timer.clone(), cfg, shutdown.signal_recv, shutdown.ack_send));

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

async fn read_input(speed_timer: Arc<Mutex<Timer>>, cfg: Config, _shutdown_send: sync::oneshot::Sender<()>) {
    // Map input key:function
    let ct = ControllableType::new();
    let mut bindings = HashMap::new();
    ct.add_bindings(&mut bindings, cfg);

    loop {
        if poll(Duration::from_secs(1)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let mut speed_timer = speed_timer.lock().unwrap();
            match read() {
                Ok(Event::Key(event)) => {
                    ct.controller_loop(event.code, &bindings, &mut speed_timer);
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

    // restore term and exit
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::Show,
        cursor::MoveTo(0, 0),
    ).unwrap();
    terminal::disable_raw_mode().unwrap();
}

async fn tick_timer(speed_timer: Arc<Mutex<Timer>>, cfg: Config, shutdown_recv: sync::watch::Receiver<bool>, _shutdown_send: sync::mpsc::UnboundedSender<()>) {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / cfg.get_ups()));

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

// TODO is there any way to separate the input logic out of main?

#[derive(Clone, Copy)]
pub struct ControllableType();

impl ControllableType {
    pub fn new() -> ControllableType {
        ControllableType()
    }
    
    fn stopstart(speed_timer: &mut Timer) {
        if speed_timer.is_running() { speed_timer.stop() } else { speed_timer.start() };
    }
    
    fn reset(speed_timer: &mut Timer) {
        speed_timer.reset();
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            Print(speed_timer.get_time_string()),
        ).expect("Reset timer failed");
    }
    
    fn split(speed_timer: &mut Timer) {
        if speed_timer.is_running() {
            speed_timer.split();
            execute!(
                stdout(),
                cursor::MoveTo(0, speed_timer.get_splits_count() + SPLITS_Y_OFFSET),
                Print(speed_timer.get_latest_split()),
            ).expect("Print split failed");
        }
    }
}

pub trait ControlHandler<T> {
    fn controller_loop(self, data: T, bindings: &HashMap<T, &dyn Fn(&mut Timer)>, speed_timer: &mut Timer);
    fn add_bindings<'a: 'b, 'b>(self, bindings: &'b mut HashMap<T, &'a dyn Fn(&mut Timer)>, cfg: Config);
}

impl ControlHandler<KeyCode> for ControllableType {
    fn controller_loop(self, data: KeyCode, bindings: &HashMap<KeyCode, &dyn Fn(&mut Timer)>, speed_timer: &mut Timer) {
        if let Some(x) = bindings.get(&data) {
            x(speed_timer)
        }
    }
    
    fn add_bindings<'a: 'b, 'b>(self, bindings: &'b mut HashMap<KeyCode, &'a dyn Fn(&mut Timer)>, cfg: Config) {
        bindings.insert(cfg.get_key_stopstart(), &ControllableType::stopstart);
        bindings.insert(cfg.get_key_reset(), &ControllableType::reset);
        bindings.insert(cfg.get_key_split(), &ControllableType::split);
    }
}
