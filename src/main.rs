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
mod network;

use game::*;
use gui::*;
use network::*;

use clap::{App, Arg, ArgMatches};
use failure::Error;
use std::sync::{Arc, Mutex};
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
            Arg::with_name("gui")
                .short("g")
                .long("gui")
                .help("Show GUI."),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase verbosity. Can be specified multiple times."),
        )
        .arg(
            Arg::with_name("address")
                .required(true)
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
    let gui = matches.is_present("gui");
    let game = Arc::new(Mutex::new(Game::new()));

    let client = EpochClient::new(&address, &name, game.clone());

    let handle = thread::spawn(move || client.run());

    if gui {
        let mut g = Gui::new((800, 600), game.clone())?;
        g.run();
    }

    handle
        .join()
        .map_err(|_| format_err!("Error while joining thread."))?;

    Ok(())
}
