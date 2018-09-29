use failure::{err_msg, Error};

use sdl2;
use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::Sdl;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use super::game::Building;

pub struct Gui<'a> {
    context: Sdl,
    canvas: WindowCanvas,
    textures: HashMap<Building, Texture<'a>>,
}

impl<'a> Gui<'a> {
    pub fn new() -> Result<Gui<'a>, Error> {
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
        Ok(Gui {
            context,
            canvas,
            textures: HashMap::new(),
        })
    }

    pub fn run(&mut self) {
        if let Err(err) = self.run_res() {
            for e in err.iter_chain() {
                error!("{}", e);
            }
        }
    }

    pub fn run_res(&mut self) -> Result<(), Error> {
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .load_texture("res/preview.jpg")
            .map_err(err_msg)?;

        let mut event_pump = self.context.event_pump().unwrap();
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
            self.canvas.clear();
            self.canvas
                .copy(&texture, None, Some(Rect::new(10, 10, 200, 200)))
                .map_err(err_msg)?;
            self.canvas.present();
        }
        Ok(())
    }
}
