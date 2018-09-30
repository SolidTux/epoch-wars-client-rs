#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

extern crate clap;
extern crate sdl2;
extern crate serde;
extern crate serde_json;
extern crate stderrlog;

mod game;
mod gui;
mod message;
mod network;

use game::*;
use gui::*;
use network::*;

use clap::{App, Arg, ArgMatches};
use failure::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn main() {
    let matches = App::new("Epoch Wars")
        .about("Client for Epoch Wars.")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true)
                .help("Player name."),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase verbosity. Can be specified multiple times."),
        )
        .arg(
            Arg::with_name("direct")
                .short("d")
                .help("Use direct connection to server instead of session server."),
        )
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .takes_value(true)
                .help("Rejoin token."),
        )
        .arg(
            Arg::with_name("address")
                .takes_value(true)
                .help("Address of server"),
        )
        .get_matches();
    stderrlog::new()
        .verbosity(matches.occurrences_of("verbosity") as usize)
        .init()
        .unwrap();

    if let Err(err) = main_res(matches) {
        for e in err.iter_chain() {
            error!("{}", e);
        }
    }
}

fn main_res(matches: ArgMatches) -> Result<(), Error> {
    let address = matches.value_of("address").unwrap_or("localhost:4200");
    let name = matches.value_of("name").unwrap_or("Noname");
    let direct = matches.is_present("direct");
    let game = Arc::new(Mutex::new(Game::new()));

    let (tx_gui, rx_net) = mpsc::channel();
    let (tx_net, rx_gui) = mpsc::channel();
    let client = EpochClient::new(
        &address,
        &name,
        matches.value_of("token"),
        tx_net,
        rx_net,
        game.clone(),
    );
    let _handle = thread::spawn(move || client.run(direct));

    let mut g = Gui::new((1024, 768), false, tx_gui, rx_gui, game.clone())?;
    g.run();
    Ok(())
}
