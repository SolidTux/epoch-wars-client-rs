use failure::Error;
use serde_json;
use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

use super::game::Game;

pub struct EpochClient {
    address: String,
    game: Arc<Mutex<Game>>,
}

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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Answer {
    Welcome { player: usize },
    EndOfTurn { scores: Vec<usize> },
    Debug { message: String },
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

impl EpochClient {
    pub fn new(address: &str, game: Arc<Mutex<Game>>) -> EpochClient {
        EpochClient {
            address: address.to_string(),
            game,
        }
    }

    pub fn run(&self) {
        if let Err(err) = self.run_res() {
            for e in err.iter_chain() {
                error!("{}", e);
            }
        }
    }

    fn run_res(&self) -> Result<(), Error> {
        debug!("Connecting to address: {}", self.address);
        let mut stream = TcpStream::connect(&self.address)?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let handle = thread::spawn(move || {
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                trace!("{}", line.trim());
                match serde_json::from_str::<Answer>(&line.trim()) {
                    Ok(a) => {
                        debug!("Answer: {:?}", a);
                        match a {
                            Answer::Debug { message: msg } => {
                                info!("Debug message from server: \n{}", msg)
                            }
                            _ => warn!("Unimplemented answer type received."),
                        }
                    }
                    Err(e) => warn!("{}", e),
                }
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
                        error!("{}", e);
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
}
