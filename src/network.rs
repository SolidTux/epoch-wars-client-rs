use failure::Error;
use serde_json;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::game::{Building, Game, ScoreEntry};
use super::message::{FromGuiMessage, ToGuiMessage};

pub struct EpochClient {
    address: String,
    name: String,
    token: Option<String>,
    game: Arc<Mutex<Game>>,
    tx: Sender<ToGuiMessage>,
    rx: Receiver<FromGuiMessage>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Command {
    Welcome { name: String },
    Rejoin { token: String },
    EndTurn,
    Build { x: u32, y: u32, building: Building },
    Excavate { x: u32, y: u32 },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Answer {
    Welcome {
        player: usize,
        map_size: (u32, u32),
        rejoin: String,
    },
    EndOfTurn {
        scores: Vec<ScoreEntry>,
        map: Vec<MapAnswer>,
        turn: u32,
        excavate_result: Option<ExcavateAnswer>,
        current_prices: HashMap<Building, u32>,
        tower_count: u32,
    },
    Error {
        message: String,
        subtype: Option<String>,
        pos: Option<(u32, u32)>,
        building: Option<Building>,
    },
    GameOver {
        message: String,
        score: i32,
    },
    Debug {
        message: String,
    },
}

#[derive(Deserialize, Debug)]
struct MapAnswer {
    pos: (u32, u32),
    building: Building,
}

#[derive(Deserialize, Debug)]
struct ExcavateAnswer {
    depth: i32,
    building: Option<Building>,
    pos: (u32, u32),
}

impl Command {
    pub fn send(&self, stream: &mut TcpStream) -> Result<(), Error> {
        let s = serde_json::to_string(self)?;
        trace!("Sending: {}", s);
        writeln!(stream, "{}", s)?;
        Ok(())
    }
}

impl EpochClient {
    pub fn new(
        address: &str,
        name: &str,
        token: Option<&str>,
        tx: Sender<ToGuiMessage>,
        rx: Receiver<FromGuiMessage>,
        game: Arc<Mutex<Game>>,
    ) -> EpochClient {
        EpochClient {
            address: address.to_string(),
            name: name.to_string(),
            token: token.map(|x| x.to_string()),
            game,
            tx,
            rx,
        }
    }

    fn listen(
        mut reader: BufReader<TcpStream>,
        tx: Sender<ToGuiMessage>,
        game: Arc<Mutex<Game>>,
    ) -> Result<(), Error> {
        let mut line = String::new();
        loop {
            if reader.read_line(&mut line)? == 0 {
                return Err(format_err!("Connection lost."));
            }
            trace!("{}", line.trim());
            match serde_json::from_str::<Answer>(&line.trim()) {
                Ok(a) => {
                    debug!("Answer: {:?}", a);
                    match a {
                        Answer::Welcome {
                            player: p,
                            map_size: s,
                            rejoin: r,
                        } => {
                            if let Ok(mut g) = game.lock() {
                                (*g).player = Some(p);
                                (*g).size = s;
                                (*g).rejoin = r.clone();
                            }
                            tx.send(ToGuiMessage::UpdateGrid)?;
                            tx.send(ToGuiMessage::Start)?;
                        }
                        Answer::EndOfTurn {
                            scores,
                            map,
                            turn,
                            excavate_result,
                            current_prices,
                            tower_count,
                        } => {
                            tx.send(ToGuiMessage::UpdateGrid)?;
                            tx.send(ToGuiMessage::UpdateBuildings)?;
                            tx.send(ToGuiMessage::ClearBuilding)?;
                            tx.send(ToGuiMessage::ClearExcavate)?;
                            if let Ok(mut g) = game.lock() {
                                (*g).scores = scores;
                                (*g).buildings.clear();
                                (*g).turn = turn;
                                (*g).prices = current_prices;
                                (*g).tower_count = tower_count;
                                for e in map {
                                    (*g).buildings.insert(e.pos, e.building);
                                }
                                if let Some(er) = excavate_result {
                                    tx.send(ToGuiMessage::ExcavateResult(
                                        er.depth,
                                        er.building,
                                        er.pos,
                                    ))?;
                                }
                            }
                        }
                        Answer::GameOver { message, score } => {
                            tx.send(ToGuiMessage::Message(
                                "Finish".to_string(),
                                format!("{}\nScore: {}", message, score),
                            ))?;
                            tx.send(ToGuiMessage::RequestQuit)?;
                        }
                        Answer::Debug { message: msg } => {
                            info!("Debug message from server: \n{}", msg)
                        }
                        Answer::Error {
                            message: msg,
                            subtype: st,
                            pos: p,
                            building: b,
                        } => {
                            info!("Error message from server: \n{}", msg);
                            tx.send(ToGuiMessage::Message("Error".to_string(), msg))?;
                            if let Some(subtype) = st {
                                match subtype.to_lowercase().as_str() {
                                    "invalidbuilderror" => tx.send(ToGuiMessage::ClearBuilding)?,
                                    "buildactionalreadyusederror" => {
                                        tx.send(ToGuiMessage::ClearBuilding)?;
                                        if let Some(pos) = p {
                                            if let Some(building) = b {
                                                tx.send(ToGuiMessage::SetBuilding(
                                                    pos,
                                                    building.clone(),
                                                ))?;
                                            }
                                        }
                                    }
                                    "gamealreadyrunning" => {
                                        tx.send(ToGuiMessage::Quit)?;
                                        return Ok(());
                                    }
                                    s => trace!("Got error subtype {}", s),
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    trace!("{:?}", e);
                    warn!("{}", e)
                }
            }
            line.clear()
        }
    }

    pub fn run(&self, direct: bool) {
        if let Err(err) = self.run_res(direct) {
            for e in err.iter_chain() {
                error!("{}", e);
            }
        }
        debug!("Network thread finished.");
    }

    fn run_res(&self, direct: bool) -> Result<(), Error> {
        debug!("Connecting to address: {}", self.address);
        let mut stream = {
            if direct {
                TcpStream::connect_timeout(
                    &self
                        .address
                        .to_socket_addrs()?
                        .next()
                        .ok_or(format_err!("Error while parsing address."))?,
                    Duration::from_millis(20000),
                )?
            } else {
                let tmp = TcpStream::connect_timeout(
                    &self
                        .address
                        .to_socket_addrs()?
                        .next()
                        .ok_or(format_err!("Error while parsing address."))?,
                    Duration::from_millis(20000),
                )?;
                let mut line = String::new();
                let mut reader = BufReader::new(tmp);
                reader.read_line(&mut line)?;
                debug!("Using server {}", line.trim());
                TcpStream::connect_timeout(
                    &line
                        .trim()
                        .to_socket_addrs()?
                        .next()
                        .ok_or(format_err!("Error while parsing address."))?,
                    Duration::from_millis(20000),
                )?
            }
        };
        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;
        debug!("Connected.");
        let reader = BufReader::new(stream.try_clone()?);
        let _ = {
            let game = self.game.clone();
            let tx = self.tx.clone();
            thread::spawn(move || EpochClient::listen(reader, tx, game))
        };
        if let Some(ref t) = self.token {
            Command::Rejoin { token: t.clone() }.send(&mut stream)?;
        } else {
            Command::Welcome {
                name: self.name.clone(),
            }.send(&mut stream)?;
        }
        while let Ok(msg) = self.rx.recv() {
            trace!("Got message from GUI: {:?}", msg);
            match msg {
                FromGuiMessage::Build(pos, building) => Command::Build {
                    x: pos.0,
                    y: pos.1,
                    building,
                }.send(&mut stream)?,
                FromGuiMessage::Excavate(pos) => {
                    Command::Excavate { x: pos.0, y: pos.1 }.send(&mut stream)?
                }
                FromGuiMessage::Skip => Command::EndTurn.send(&mut stream)?,
                FromGuiMessage::Quit => break,
            }
        }
        Ok(())
    }
}
