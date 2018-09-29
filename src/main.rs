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
mod network;

use game::*;
use network::*;

use clap::{App, Arg, ArgMatches};
use failure::{err_msg, Error};
use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let matches = App::new("Epoch Wars")
        .about("Client for Epoch Wars.")
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
    let gui = matches.is_present("gui");
    let game = Arc::new(Mutex::new(Game::new()));

    let client = EpochClient::new(&address, game.clone());

    let handle = thread::spawn(move || client.run());

    if gui {
        sdl()?;
    }

    handle
        .join()
        .map_err(|_| format_err!("Error while joining thread."))?;

    Ok(())
}

fn sdl() -> Result<(), Error> {
    let context = sdl2::init().map_err(err_msg)?;
    let video = context.video().map_err(err_msg)?;
    let _image_context = sdl2::image::init(INIT_PNG | INIT_JPG).map_err(err_msg)?;

    let window = video
        .window("Epoch Wars", 800, 600)
        .opengl()
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::RGB(0, 255, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .load_texture("res/preview.jpg")
        .map_err(err_msg)?;

    let mut event_pump = context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        canvas.clear();
        canvas
            .copy(&texture, None, Some(Rect::new(10, 10, 200, 200)))
            .map_err(err_msg)?;
        canvas.present();
    }

    Ok(())
}
