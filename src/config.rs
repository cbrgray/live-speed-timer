use crossterm::event::KeyCode;
use serde::{Serialize, Deserialize};
use serde_yaml;

use std::fs;
use std::clone::Clone;
use std::io::{Write, Read};

#[derive(Serialize, Deserialize)]
#[serde(remote = "crossterm::event::KeyCode")]
enum KeyCodeDef {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Keys {
    #[serde(with = "KeyCodeDef")]
    split: KeyCode,
    #[serde(with = "KeyCodeDef")]
    stopstart: KeyCode,
    #[serde(with = "KeyCodeDef")]
    reset: KeyCode,
    #[serde(with = "KeyCodeDef")]
    quit: KeyCode,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    #[serde(default = "default_ups", rename = "updates_per_second")]
    ups: u64,
    #[serde(default = "default_keys")]
    keys: Keys,
}

fn default_ups() -> u64 {
    return 30;
}

fn default_keys() -> Keys {
    return Keys {
        split: KeyCode::Char(' '),
        stopstart: KeyCode::Char('s'),
        reset: KeyCode::Char('r'),
        quit: KeyCode::Esc,
    }
}

impl Config {
    fn new() -> Config {
        Config {
            ups: default_ups(),
            keys: default_keys(),
        }
    }

    pub fn load_config(filepath: &str) -> Config {
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filepath);
        
        if file.is_ok() {
            // file was created successfully, therefore it didn't exist, so populate it with defaults
            let def = Config::new();
            let def_str = serde_yaml::to_string(&def).unwrap();
            file.unwrap().write_all(def_str.as_bytes()).expect("Failed to create new cfg file");
            return def;
        }

        // file couldn't be created so it did exist - just read it normally
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(filepath);

        let mut contents = String::new();
        file.unwrap().read_to_string(&mut contents).expect("Failed to read cfg file");

        return serde_yaml::from_str(&contents).expect("Failed to load cfg");
    }

    pub fn get_ups(self) -> u64 {
        return self.ups;
    }

    pub fn get_key_split(self) -> KeyCode {
        return self.keys.split;
    }

    pub fn get_key_stopstart(self) -> KeyCode {
        return self.keys.stopstart;
    }

    pub fn get_key_reset(self) -> KeyCode {
        return self.keys.reset;
    }

    pub fn get_key_quit(self) -> KeyCode {
        return self.keys.quit;
    }
}
