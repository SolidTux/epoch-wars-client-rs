use failure::{err_msg, Error};

use sdl2;
use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::game::{Building, Game};

pub struct Gui {
    game: Arc<Mutex<Game>>,
    size: (u32, u32),
    context: Sdl,
    ttf_context: Sdl2TtfContext,
    canvas: WindowCanvas,
    assets: Assets,
}

struct Assets {
    buildings: HashMap<Building, Sprite>,
    font: String,
    background: Sprite,
}

#[derive(Clone)]
struct Sprite {
    size: u32,
    path: String,
}

impl Assets {
    pub fn new() -> Assets {
        let buildings: HashMap<Building, Sprite> = [
            (
                Building::House,
                Sprite {
                    size: 0,
                    path: "res/house.png".to_string(),
                },
            ),
            (
                Building::Villa,
                Sprite {
                    size: 1,
                    path: "res/villa.png".to_string(),
                },
            ),
            (
                Building::Tower,
                Sprite {
                    size: 0,
                    path: "res/tower.png".to_string(),
                },
            ),
        ].iter()
            .cloned()
            .collect();
        Assets {
            buildings,
            font: "res/font.ttf".to_string(),
            background: Sprite {
                size: 1,
                path: "res/bg.png".to_string(),
            },
        }
    }
}

impl Gui {
    pub fn new(size: (u32, u32), game: Arc<Mutex<Game>>) -> Result<Gui, Error> {
        let context = sdl2::init().map_err(err_msg)?;
        let video = context.video().map_err(err_msg)?;
        let _image_context = sdl2::image::init(INIT_PNG | INIT_JPG).map_err(err_msg)?;
        let ttf_context = sdl2::ttf::init().map_err(err_msg)?;

        let assets = Assets::new();

        let window = video
            .window("Epoch Wars", size.0, size.1)
            .position_centered()
            .build()?;

        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(Color::RGB(0, 255, 0));
        canvas.clear();
        canvas.present();
        Ok(Gui {
            game,
            size,
            context,
            ttf_context,
            canvas,
            assets,
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
        let bg_texture = texture_creator
            .load_texture(&self.assets.background.path)
            .map_err(err_msg)?;
        let font = self
            .ttf_context
            .load_font(&self.assets.font, 60)
            .map_err(err_msg)?;
        let mut event_pump = self.context.event_pump().unwrap();
        let mut counter = 0;
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
            self.canvas.set_draw_color(Color::RGB(50, 50, 50));
            self.canvas.clear();
            if let Ok(game) = self.game.lock() {
                let (w, h) = self.size;
                let (nx, ny) = game.size;
                let x = (w as f64) / (nx as f64);
                let y = (h as f64) / (ny as f64);
                let s = x.min(y).round() as u32;
                let x_min = (w - s * nx);
                let y_min = (h - s * ny) / 2;
                for xt in 0..nx {
                    for yt in 0..ny {
                        let r = Rect::new((x_min + s * xt) as i32, (y_min + s * yt) as i32, s, s);
                        self.canvas
                            .copy(&bg_texture, None, Some(r))
                            .map_err(err_msg)?;
                    }
                }
                for (pos, building) in &game.buildings {
                    let texture = texture_creator
                        .load_texture(&self.assets.buildings[&building].path)
                        .map_err(err_msg)?;
                    let bs = &self.assets.buildings[&building].size;
                    let r = Rect::new(
                        (x_min + s * (pos.0 - bs)) as i32,
                        (y_min + s * (pos.1 - bs)) as i32,
                        s * (1 + 2 * bs),
                        s * (1 + 2 * bs),
                    );
                    self.canvas.copy(&texture, None, Some(r)).map_err(err_msg)?;
                }
                let mut strings = Vec::new();
                let mut f = ::std::f64::INFINITY;
                let mut h = 0;
                for score in &game.scores {
                    let s = format!("{:3}: {}", score.score, score.name);
                    let surf = font
                        .render(&s)
                        .blended(Color::RGB(255, 255, 255))
                        .map_err(err_msg)?;
                    let text = texture_creator.create_texture_from_surface(&surf).unwrap();
                    let mut r = surf.rect();
                    f = f.min((x_min as f64) / (r.w as f64));
                    h = h.max(r.h);
                    strings.push(s);
                }
                for (i, s) in strings.iter().enumerate() {
                    if s.len() > 0 {
                        let surf = font
                            .render(&s)
                            .blended(Color::RGB(255, 255, 255))
                            .map_err(err_msg)?;
                        let text = texture_creator.create_texture_from_surface(&surf).unwrap();
                        let mut r = surf.rect();
                        r.y += h * (i as i32);
                        let f = (x_min as f64) / (r.w as f64);
                        r.w = ((r.w as f64) * f).round() as i32;
                        r.h = ((r.h as f64) * f).round() as i32;
                        self.canvas.copy(&text, None, Some(r)).map_err(err_msg)?;
                    }
                }
            }
            counter = counter + 1;
            self.canvas.present();
        }
        Ok(())
    }
}
