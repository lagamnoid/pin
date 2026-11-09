#![allow(unused)]
use std::env::args;
use std::io::{self, Write};
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
pin show <idx> - show full content of not for cwd at index
pin del <idx> - delete note for cwd at index
pin clear - delete all notes for cwd
pin purge - delete all notes for all directories
pin global - list notes for all directories
pin move <src> <dest> - move notes from one directory to another
pin rmdead <-c/--confirm (optional)> - delete notes for directoties that no longer exist (use -c to ask for confirmation for every directory)
";


fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    // Reserve 3 chars for "..."
    let take = max_len.saturating_sub(3);
    let mut result: String = s.chars().take(take).collect();
    result.push_str("...");
    result
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pin {
    content: String,
    datetime: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
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
            println!("#{}        {:<54}[{}]", i+1, truncate(&p.content, 50), p.datetime);
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

    fn show_pin(&self, location: &str, index: usize) {
        if !(self.pins.contains_key(location)) {
            println!("No pins for current directory");
            return;
        }
        if index - 1 > self.pins.get(location).unwrap().len() - 1 {
            println!("Error: index out of range");
            return;
        }
        println!("{}", self.pins.get(location).unwrap()[index - 1].content);
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

    fn rmdead(&mut self, confirm: bool) {
        for (loc, v) in self.pins.clone() {
            let path = PathBuf::from(&loc);
            println!("Checking path: {}", loc);
            if !(path.exists()) {
                if confirm {
                    let mut e = String::new();
                    println!("Path '{}' does not exist.", loc);
                    print!("Remove path? [y/n]: ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut e);
                    let eman = e.trim_end();
                    if eman == "y" {
                        self.pins.remove(&loc);
                        println!("Path removed");
                    } else {
                        println!("Deletion canceled");
                    }
                } else {
                    println!("Path '{}' does not exist and has been removed.", loc);
                    self.pins.remove(&loc);
                }
            }
        }
    }

    fn move_notes(&mut self, src: String, dest: String) {
        let mut pinmen = self.pins.get(&src).unwrap_or_else(|| {
            eprintln!("Error: path '{}' has no notes", src);
            process::exit(1);
        }).to_vec();
        if self.pins.contains_key(&dest) {
            self.pins.get_mut(&dest).unwrap().append(&mut pinmen);
        } else {
            self.pins.insert(dest, pinmen);
        }
        self.pins.remove(&src);
        
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
    let argc = args.len();
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
            if argc < 3 {
                eprintln!("Error: missing argument");
                process::exit(1)
            }
            else {
                config.add_pin(&cwd, args[2].clone(), get_datetime());
            }
        },
        "del" => {
            if argc < 3 {
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
        "rmdead" => config.rmdead(argc >= 3 && (args[2] == "-c" || args[2] == "--confirm")),
        "move" => {
            if argc < 3 {
                eprintln!("Error: missing arguments");
                process::exit(1)
            }
            let src: String = if args[2].starts_with('\\') {
                args[2].clone()
            } else {
                cwd.clone() + "/" + &args[2]
            };
            let dest: String = if args[3].starts_with('\\') {
                args[3].clone()
            } else {
                cwd.clone() + "/" + &args[3]
            };
            config.move_notes(src, dest);
        },
        "show" => {
            if argc < 3 {
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
            config.show_pin(&cwd, num);
        },
        _ => eprintln!("Error: incorrect arguments (try pin help)"),
    }
    config.write(&path);

}
