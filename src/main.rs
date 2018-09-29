#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

extern crate sdl2;
extern crate serde;
extern crate serde_json;

use failure::{err_msg, Error};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

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
    let context = sdl2::init().map_err(err_msg)?;
    let video = context.video().map_err(err_msg)?;

    let window = video
        .window("Epoch Wars", 800, 600)
        .opengl()
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::RGB(0, 255, 0));
    canvas.clear();
    canvas.present();

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
        canvas.present();
    }

    Ok(())
}
