use failure::{err_msg, Error};

use sdl2;
use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::Sdl;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::game::{Building, Game};
use super::message::{FromGuiMessage, ToGuiMessage};

pub struct Gui {
    game: Arc<Mutex<Game>>,
    size: (u32, u32),
    context: Sdl,
    ttf_context: Sdl2TtfContext,
    canvas: WindowCanvas,
    assets: Assets,
    active: usize,
    tx: Sender<FromGuiMessage>,
    rx: Receiver<ToGuiMessage>,
    running: bool,
}

struct Assets {
    buildings: HashMap<Building, Sprite>,
    font: String,
    background: Sprite,
    excavation: Sprite,
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
            excavation: Sprite {
                size: 1,
                path: "res/ex.png".to_string(),
            },
        }
    }
}

impl Gui {
    pub fn new(
        size: (u32, u32),
        fullscreen: bool,
        tx: Sender<FromGuiMessage>,
        rx: Receiver<ToGuiMessage>,
        game: Arc<Mutex<Game>>,
    ) -> Result<Gui, Error> {
        let context = sdl2::init().map_err(err_msg)?;
        let video = context.video().map_err(err_msg)?;
        let _image_context = sdl2::image::init(INIT_PNG | INIT_JPG).map_err(err_msg)?;
        let ttf_context = sdl2::ttf::init().map_err(err_msg)?;

        let assets = Assets::new();

        let mut window_builder = video.window("Epoch Wars", size.0, size.1);
        if fullscreen {
            window_builder.fullscreen();
        }
        let window = window_builder.position_centered().build()?;

        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(Color::RGB(0, 255, 0));
        canvas.clear();
        canvas.present();
        Ok(Gui {
            game,
            active: 0,
            size,
            context,
            ttf_context,
            canvas,
            assets,
            tx,
            rx,
            running: false,
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
        let ex_texture = texture_creator
            .load_texture(&self.assets.excavation.path)
            .map_err(err_msg)?;
        let font = self
            .ttf_context
            .load_font(&self.assets.font, 60)
            .map_err(err_msg)?;
        let mut event_pump = self.context.event_pump().unwrap();
        let (w, h) = self.size;
        let (mut nx, mut ny, mut s, mut x_min, mut y_min, ew, eg) = {
            let game = self
                .game
                .lock()
                .map_err(|_| format_err!("Error while locking Mutex."))?;
            let (nx, ny) = game.size;
            let x = (w as f64) / (nx as f64);
            let y = (h as f64) / (ny as f64);
            let s = x.min(y).round() as u32;
            let x_min = w - s * nx;
            let y_min = (h - s * ny) / 2;
            let ew = (x_min * 2 / 9) as i32;
            let eg = (ew * 1 / 10) as i32;
            (nx, ny, s, x_min, y_min, ew, eg)
        };

        let mut excavation_position = None;
        let mut temp_building = None;
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        self.tx.send(FromGuiMessage::Quit)?;
                        break 'running;
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Right,
                        x,
                        y,
                        ..
                    } => {
                        let gx = (x - (x_min as i32)) / (s as i32);
                        let gy = (y - (y_min as i32)) / (s as i32);
                        if (gx >= 0) && (gx < nx as i32) && (gy >= 0) && (gy < ny as i32) {
                            let gx = gx as u32;
                            let gy = gy as u32;
                            self.tx.send(FromGuiMessage::Excavate((gx, gy)))?;
                            excavation_position = Some((gx, gy));
                        }
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        x,
                        y,
                        ..
                    } => {
                        let gx = (x - (x_min as i32)) / (s as i32);
                        let gy = (y - (y_min as i32)) / (s as i32);
                        let building = match self.active {
                            0 => Building::House,
                            1 => Building::Villa,
                            2 => Building::Tower,
                            _ => Building::House,
                        };
                        let bs = self.assets.buildings[&building].size as i32;
                        if (x < (x_min as i32)) && (y > ((h as i32) - ew)) {
                            self.active = ((x / ew) as usize).min(2);
                            debug!("Element {} active.", self.active);
                        } else if (gx >= bs)
                            && (gx < (nx as i32 - bs))
                            && (gy >= bs)
                            && (gy < (nx as i32 - bs))
                        {
                            let gx = gx as u32;
                            let gy = gy as u32;
                            temp_building = Some(((gx, gy), building.clone()));
                            self.tx.send(FromGuiMessage::Build((gx, gy), building))?;
                        }
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            self.canvas.set_draw_color(Color::RGB(50, 50, 50));
            self.canvas.clear();
            if self.running {
                for (i, building) in [Building::House, Building::Villa, Building::Tower]
                    .iter()
                    .enumerate()
                {
                    let texture = texture_creator
                        .load_texture(&self.assets.buildings[&building].path)
                        .map_err(err_msg)?;
                    let r = Rect::new(
                        (i as i32) * ew + eg,
                        (h as i32) - ew - eg,
                        (ew - 2 * eg) as u32,
                        (ew - 2 * eg) as u32,
                    );
                    if i == self.active {
                        self.canvas.set_draw_color(Color::RGB(255, 0, 0));
                        self.canvas.fill_rect(r).map_err(err_msg)?;
                    }
                    self.canvas.copy(&texture, None, Some(r)).map_err(err_msg)?;
                }
                if let Ok(game) = self.game.lock() {
                    nx = game.size.0;
                    ny = game.size.1;
                    let x = (w as f64) / (nx as f64);
                    let y = (h as f64) / (ny as f64);
                    s = x.min(y).round() as u32;
                    x_min = w - s * nx;
                    y_min = (h - s * ny) / 2;
                    for xt in 0..nx {
                        for yt in 0..ny {
                            let r =
                                Rect::new((x_min + s * xt) as i32, (y_min + s * yt) as i32, s, s);
                            if excavation_position
                                .map(|(x, y)| (x == xt) && (y == yt))
                                .unwrap_or(false)
                            {
                                self.canvas
                                    .copy(&ex_texture, None, Some(r))
                                    .map_err(err_msg)?;
                            } else {
                                self.canvas
                                    .copy(&bg_texture, None, Some(r))
                                    .map_err(err_msg)?;
                            }
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
                    if let Some((pos, building)) = &temp_building {
                        let mut texture = texture_creator
                            .load_texture(&self.assets.buildings[&building].path)
                            .map_err(err_msg)?;
                        texture.set_alpha_mod(127);
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
                        let mut r = surf.rect();
                        f = f.min((x_min as f64) / (r.w as f64));
                        h = h.max((r.h as f64).round() as i32);
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
                            r.w = ((r.w as f64) * f).round() as i32;
                            r.h = ((r.h as f64) * f).round() as i32;
                            self.canvas.copy(&text, None, Some(r)).map_err(err_msg)?;
                        }
                    }
                }
            }
            self.canvas.present();
            if let Ok(msg) = self.rx.try_recv() {
                match msg {
                    ToGuiMessage::Start => self.running = true,
                    ToGuiMessage::Message(t, s) => {
                        show_simple_message_box(
                            MessageBoxFlag::empty(),
                            &t,
                            &s,
                            self.canvas.window(),
                        )?;
                    }
                    ToGuiMessage::ExcavateResult(d, b, p) => match b {
                        Some(building) => show_simple_message_box(
                            MessageBoxFlag::empty(),
                            "Excavation Results",
                            &format!(
                                "Found {:?} at depth {} on position {}, {}.",
                                building, d, p.0, p.1
                            ),
                            self.canvas.window(),
                        )?,
                        None => show_simple_message_box(
                            MessageBoxFlag::empty(),
                            "Excavation Results",
                            &format!("Found nothing at position {}, {}.", p.0, p.1),
                            self.canvas.window(),
                        )?,
                    },
                    ToGuiMessage::ClearBuilding => temp_building = None,
                    ToGuiMessage::SetBuilding(pos, building) => {
                        temp_building = Some((pos, building.clone()))
                    }
                    ToGuiMessage::ClearExcavate => excavation_position = None,
                    ToGuiMessage::Quit => break 'running,
                }
            }
        }
        Ok(())
    }
}
