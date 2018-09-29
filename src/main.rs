#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::net::TcpStream;
use std::thread;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum Command {
    Build {
        x: usize,
        y: usize,
        building: String,
    },
}

impl Command {
    pub fn from_line(s: &str) -> Option<Command> {
        let parts = s.trim().split(' ').collect::<Vec<&str>>();
        match parts.get(0) {
            Some(&"build") => {
                if parts.len() < 4 {
                    None
                } else {
                    Some(Command::Build {
                        x: parts[1].parse().ok()?,
                        y: parts[2].parse().ok()?,
                        building: parts[3].to_string(),
                    })
                }
            }
            _ => None,
        }
    }
}

fn main() {
    let mut stream = TcpStream::connect("localhost:4200").unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let handle = thread::spawn(move || {
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok() {
            print!("{}", line.replace(";", "\n"));
            line.clear()
        }
    });

    let mut reader = BufReader::new(stdin());
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok() {
        if let Some(cmd) = Command::from_line(&line) {
            let s = serde_json::to_string(&cmd).unwrap();
            println!("{}", s);
            writeln!(&mut stream, "{}", s);
        }
        line.clear();
    }

    handle.join().unwrap();
}
