use serde::{Serialize, Deserialize};
use serde_yaml;

use std::fs;
use std::clone::Clone;
use std::io::{Write, Read};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    #[serde(default = "default_ups", alias = "updates_per_second")]
    ups: u64,
}

fn default_ups() -> u64 {
    return 30;
}

impl Config {
    fn new() -> Config {
        Config {
            ups: default_ups(),
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
}
