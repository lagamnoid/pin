#![allow(unused)]
use std::env::args;
use std::io;
use std::env;
use std::fs::File;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::process;
use chrono::Local;

const HELP_MESSAGE: &str = "pin - list notes for cwd
pin add \"<text>\" - add note to current directory
pin del <idx> - delete note for cwd at index
pin clear - delete all notes for cwd
pin purge - delete all notes for all directories
pin global - list notes for all directories
pin move <src> <dest> - move notes from one directory to another
pin rmdead - delete notes for directoties that no longer exist
";


#[derive(Debug, Serialize, Deserialize)]
struct Pin {
    content: String,
    datetime: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct Config {
    pins: HashMap<String, Vec<Pin>>,
}

impl Config {
    fn add_pin(&mut self, location: &str, content: String, datetime: String) {
        let new_pin = Pin { content, datetime };
        self.pins
            .entry(location.to_string())
            .or_default()
            .push(new_pin);
    }

    fn write(&self, path: &PathBuf) {
        let file = File::create(path).expect("Failed to write config");
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).expect("Failed to write config");
    }
    
    fn list(&self, location: &str) {
        if !(self.pins.contains_key(location)) {
            println!("No pins for current directory");
            return;
        }
        let pinmen = self.pins.get(location).unwrap();
        for (i, p) in pinmen.iter().enumerate() {
            println!("#{}        {}        [{}]", i+1, p.content, p.datetime);
        }
    }

    fn del_pin(&mut self, location: &str, index: usize) {
        if !(self.pins.contains_key(location)) {
            println!("No pins for current directory");
            return;
        }
        if index - 1 > self.pins.get(location).unwrap().len() - 1 {
            println!("Error: index out of range");
            return;
        }
        self.pins.get_mut(location).unwrap().remove(index - 1);
    }

    fn clear(&mut self, location: &str) {
        if !(self.pins.contains_key(location)) {
            return;
        } 
        self.pins.remove(location);
    }

    fn purge(&mut self) {
        self.pins.clear();
    }

    fn global(&self) {
        for (loc, v) in &self.pins {
            println!("    => {}:", loc);
            self.list(loc);
        }
    }

}




fn load_config(path: &PathBuf) -> Config {
    let file = File::open(path).unwrap_or_else(|_| {
        eprintln!("Error: failed to open file");
        process::exit(1);
    });
    let reader = BufReader::new(file);
    let mut config: Config = serde_json::from_reader(reader).unwrap_or_else(|_| {
        eprintln!("Error: failed to load jason");
        process::exit(1);
    });
    config
}


fn get_path() -> PathBuf {
    let home_env = env::var("HOME").unwrap_or_else(|_| {
        eprintln!("Error: $HOME environment variable is not set.");
        process::exit(1);
    });
    let mut path = PathBuf::from(home_env);
    path.push(".config/pin/data.json");
    path
}

fn get_datetime() -> String {
    let now = Local::now();
    now.format("%m-%d %H:%M").to_string()
}

fn check_existtown(path: &PathBuf) {
    if !(path.exists()) {
        fs::write(path, "{\"pins\": {}}");
    }
}


fn main() {
    let cwd: String = env::current_dir().expect("failed to get cwd").to_string_lossy().into_owned();
    let args: Vec<String> = env::args().collect();
    let path = get_path();
    check_existtown(&path);
    let mut config: Config = load_config(&path);
    if args.len() < 2 {
        config.list(&cwd);
        process::exit(0);
    }
    // config.add_pin(String::from("emnoi"), String::from("emnoi"), String::from("emnoi"));
    // config.write(&path);
    match args[1].as_str() {
        "help" => {
            print!("{}", HELP_MESSAGE);
        }
        "add" => {
            if args.len() < 3 {
                eprintln!("Error: missing argument");
                process::exit(1)
            }
            else {
                config.add_pin(&cwd, args[2].clone(), get_datetime());
            }
        },
        "del" => {
            if args.len() < 3 {
                eprintln!("Error: missing argument");
                process::exit(1)
            }
            let num: usize = match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("Error: index must be an integer");
                    std::process::exit(1);
                }
            };
            config.del_pin(&cwd, num);
        },
        "clear" => config.clear(&cwd),
        "purge" => config.purge(),
        "global" => config.global(),
        _ => eprintln!("Error: incorrect arguments (try pin help)"),
    }
    config.write(&path);

}

