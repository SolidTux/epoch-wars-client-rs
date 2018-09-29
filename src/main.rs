#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

extern crate serde;
extern crate serde_json;

use failure::Error;
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
    Excavate {
        x: usize,
        y: usize,
    },
}

impl Command {
    pub fn from_line(s: &str) -> Result<Command, Error> {
        let parts = s.trim().split(' ').collect::<Vec<&str>>();
        match parts.get(0) {
            Some(&"b") => {
                if parts.len() < 4 {
                    Err(format_err!("Not enough arguments."))
                } else {
                    Ok(Command::Build {
                        x: parts[1].parse()?,
                        y: parts[2].parse()?,
                        building: parts[3].to_string(),
                    })
                }
            }
            Some(&"e") => {
                if parts.len() < 3 {
                    Err(format_err!("Not enough arguments."))
                } else {
                    Ok(Command::Excavate {
                        x: parts[1].parse()?,
                        y: parts[2].parse()?,
                    })
                }
            }
            Some(s) => Err(format_err!("Unknown command \"{}\".", s)),
            None => Err(format_err!("No command specified.")),
        }
    }
}

fn main() {
    if let Err(err) = main_res() {
        for e in err.iter_chain() {
            eprintln!("{}", e);
        }
    }
}

fn main_res() -> Result<(), Error> {
    let mut stream = TcpStream::connect("localhost:4200")?;
    let mut reader = BufReader::new(stream.try_clone()?);
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
        match Command::from_line(&line) {
            Ok(cmd) => {
                let s = serde_json::to_string(&cmd)?;
                println!("{}", s);
                writeln!(&mut stream, "{}", s)?;
            }
            Err(err) => {
                for e in err.iter_chain() {
                    eprintln!("{}", e);
                }
            }
        }
        line.clear();
    }

    handle
        .join()
        .map_err(|_| format_err!("Error while joining thread."))?;
    Ok(())
}
