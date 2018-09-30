use failure::{err_msg, Error};

use sdl2;
use sdl2::event::Event;
use sdl2::image::{LoadTexture, INIT_JPG, INIT_PNG};
use sdl2::keyboard::Keycode;
use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::WindowContext;
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
    active: Vec<Sprite>,
}

#[derive(Clone)]
struct Sprite {
    size: u32,
    path: String,
    building: Option<Building>,
    index: Option<(u32, u32)>,
    rect: Option<Rect>,
}

impl Sprite {
    pub fn new(size: u32, path: &str) -> Sprite {
        Sprite {
            size,
            path: path.to_string(),
            building: None,
            index: None,
            rect: None,
        }
    }

    pub fn contains(&self, pos: (i32, i32)) -> bool {
        self.rect.map(|x| x.contains_point(pos)).unwrap_or(false)
    }

    pub fn draw(
        &self,
        texture_creator: &TextureCreator<WindowContext>,
        canvas: &mut WindowCanvas,
    ) -> Result<(), Error> {
        let texture = texture_creator.load_texture(&self.path).map_err(err_msg)?;
        canvas.copy(&texture, None, self.rect).map_err(err_msg)?;
        Ok(())
    }

    pub fn draw_alpha(
        &self,
        texture_creator: &TextureCreator<WindowContext>,
        canvas: &mut WindowCanvas,
        alpha: u8,
    ) -> Result<(), Error> {
        let mut texture = texture_creator.load_texture(&self.path).map_err(err_msg)?;
        texture.set_alpha_mod(alpha);
        canvas.copy(&texture, None, self.rect).map_err(err_msg)?;
        Ok(())
    }
}

impl Assets {
    pub fn new() -> Assets {
        let buildings: HashMap<Building, Sprite> = [
            (Building::House, Sprite::new(0, "res/house.png")),
            (Building::Villa, Sprite::new(1, "res/villa.png")),
            (Building::Tower, Sprite::new(0, "res/tower.png")),
        ].iter()
            .cloned()
            .collect();
        Assets {
            buildings,
            font: "res/font.ttf".to_string(),
            background: Sprite::new(0, "res/bg.png"),
            excavation: Sprite::new(0, "res/ex.png"),
            active: vec![
                Sprite::new(0, "res/house.png"),
                Sprite::new(1, "res/villa.png"),
                Sprite::new(0, "res/tower.png"),
                Sprite::new(0, "res/skip.png"),
            ],
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

        let mut assets = Assets::new();
        assets.active[0].building = Some(Building::House);
        assets.active[1].building = Some(Building::Villa);
        assets.active[2].building = Some(Building::Tower);

        let mut window_builder = video.window("Epoch Wars", size.0, size.1);
        if fullscreen {
            window_builder.fullscreen_desktop();
        }
        let window = window_builder.position_centered().build()?;

        let mut canvas = window.into_canvas().build()?;

        canvas.set_draw_color(Color::RGB(0, 255, 0));
        canvas.clear();
        canvas.present();
        Ok(Gui {
            game,
            active: 0,
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
        let font = self
            .ttf_context
            .load_font(&self.assets.font, 60)
            .map_err(err_msg)?;
        let mut event_pump = self.context.event_pump().unwrap();

        let mut excavation_sprite = None;
        let mut temp_sprite: Option<Sprite> = None;
        let mut mouse_pos = (0, 0);
        let mut building_sprites: Vec<Sprite> = Vec::new();
        let mut grid_sprites: Vec<Sprite> = Vec::new();
        'running: loop {
            let (w, h) = self.canvas.window().drawable_size();
            let (nx, ny) = {
                let game = self
                    .game
                    .lock()
                    .map_err(|_| format_err!("Error while locking Mutex."))?;
                game.size
            };
            let x = (w as f64) / (nx as f64);
            let y = (h as f64) / (ny as f64);
            let s = x.min(y).floor() as u32;
            let x_min = w - s * nx;
            let y_min = (h - s * ny) / 2;
            let ew = (x_min * 2 / 9) as i32;
            let eg = (ew * 1 / 10) as i32;
            let ag = (ew * 4 / 10) as i32;
            for i in 0..4 {
                self.assets.active[i].rect = Some(Rect::new(
                    eg,
                    (h as i32) + (i as i32) * ew - eg - 4 * ew,
                    (ew - 2 * eg) as u32,
                    (ew - 2 * eg) as u32,
                ));
            }

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
                        for sprite in &grid_sprites {
                            if sprite.contains((x, y)) {
                                if let Some(pos) = sprite.index {
                                    let mut s = self.assets.excavation.clone();
                                    s.index = Some(pos);
                                    s.rect = sprite.rect.clone();
                                    excavation_sprite = Some(s);
                                    self.tx.send(FromGuiMessage::Excavate(pos))?;
                                }
                            }
                        }
                    }
                    Event::MouseButtonUp {
                        mouse_btn: MouseButton::Left,
                        x,
                        y,
                        ..
                    } => {
                        for (i, sprite) in self.assets.active.iter().enumerate() {
                            if sprite.contains((x, y)) {
                                if i < 3 {
                                    self.active = i;
                                } else if i == 3 {
                                    self.tx.send(FromGuiMessage::Skip)?;
                                }
                            }
                        }
                        for sprite in &grid_sprites {
                            if sprite.contains((x, y)) {
                                if let Some(pos) = sprite.index {
                                    let asprite = self.assets.active[self.active].clone();
                                    if let Some(building) = asprite.building {
                                        let bs = asprite.size;
                                        temp_sprite = Some(Sprite {
                                            size: bs,
                                            index: Some(pos),
                                            path: self.assets.buildings[&building].path.clone(),
                                            building: Some(building.clone()),
                                            rect: Some(Rect::new(
                                                (x_min + s * (pos.0 - bs)) as i32,
                                                (y_min + s * (pos.1 - bs)) as i32,
                                                s * (1 + 2 * bs),
                                                s * (1 + 2 * bs),
                                            )),
                                        });
                                        self.tx.send(FromGuiMessage::Build(pos, building.clone()))?;
                                    }
                                }
                            }
                        }
                    }
                    Event::MouseMotion { x, y, .. } => mouse_pos = (x, y),
                    _ => {}
                }
            }
            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            self.canvas.set_draw_color(Color::RGB(50, 50, 50));
            self.canvas.clear();
            if self.running {
                for sprite in &grid_sprites {
                    sprite.draw(&texture_creator, &mut self.canvas)?;
                    if let Some(r) = sprite.rect {
                        if sprite.contains(mouse_pos) {
                            self.canvas.set_draw_color(Color::RGB(255, 0, 0));
                            let r = r.clone();
                            self.canvas.draw_rect(r).map_err(err_msg)?;
                        }
                    }
                }
                if let Some(sprite) = &excavation_sprite {
                    sprite.draw(&texture_creator, &mut self.canvas)?;
                }
                if let Ok(game) = self.game.lock() {
                    for (i, sprite) in self.assets.active.iter().enumerate() {
                        if let Some(r) = sprite.rect {
                            if i == self.active {
                                self.canvas.set_draw_color(Color::RGB(255, 0, 0));
                                let r = r.clone();
                                self.canvas.fill_rect(r).map_err(err_msg)?;
                            } else if sprite.contains(mouse_pos) {
                                self.canvas.set_draw_color(Color::RGB(255, 0, 0));
                                let r = r.clone();
                                self.canvas.draw_rect(r).map_err(err_msg)?;
                            }
                            if i < 3 {
                                if let Some(building) = &sprite.building {
                                    if let Some(price) = game.prices.get(building) {
                                        let s = if i == 2 {
                                            format!("{} ({})", price, game.tower_count)
                                        } else {
                                            format!("{}", price)
                                        };
                                        let surf = font
                                            .render(&s)
                                            .blended(Color::RGB(255, 255, 255))
                                            .map_err(err_msg)?;
                                        let text = texture_creator
                                            .create_texture_from_surface(&surf)
                                            .unwrap();
                                        let mut rt = surf.rect().clone();
                                        rt.x = ew + 3 * eg;
                                        rt.y = r.y;
                                        rt.w = (rt.w * (ew - 2 * eg)) / rt.h;
                                        rt.h = ew - 2 * eg;
                                        self.canvas.copy(&text, None, Some(rt)).map_err(err_msg)?;
                                    }
                                }
                            }
                        }
                        sprite.draw(&texture_creator, &mut self.canvas)?
                    }
                    if let Some(sprite) = &temp_sprite {
                        sprite.draw_alpha(&texture_creator, &mut self.canvas, 100)?;
                    }
                    let mut strings = vec![format!("Turn {}", game.turn)];
                    let mut f = ::std::f64::INFINITY;
                    let mut h = 0;
                    for sprite in &building_sprites {
                        sprite.draw(&texture_creator, &mut self.canvas)?;
                    }
                    for score in &game.scores {
                        let s = format!("{:3}: {}", score.score, score.name);
                        let surf = font
                            .render(&s)
                            .blended(Color::RGB(255, 255, 255))
                            .map_err(err_msg)?;
                        let mut r = surf.rect();
                        f = f.min((x_min as f64 - ag as f64) / (r.w as f64));
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
                            r.x = eg;
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
                trace!("Got message: {:?}", msg);
                match msg {
                    ToGuiMessage::Start => self.running = true,
                    ToGuiMessage::Message(t, s) => show_simple_message_box(
                        MessageBoxFlag::empty(),
                        &t,
                        &s,
                        self.canvas.window(),
                    )?,
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
                    ToGuiMessage::ClearBuilding => temp_sprite = None,
                    ToGuiMessage::SetBuilding(pos, building) => {
                        let bs = self.assets.buildings[&building].size;
                        temp_sprite = Some(Sprite {
                            size: bs,
                            index: Some(pos),
                            path: self.assets.buildings[&building].path.clone(),
                            building: Some(building.clone()),
                            rect: Some(Rect::new(
                                (x_min + s * (pos.0 - bs)) as i32,
                                (y_min + s * (pos.1 - bs)) as i32,
                                s * (1 + 2 * bs),
                                s * (1 + 2 * bs),
                            )),
                        })
                    }
                    ToGuiMessage::ClearExcavate => excavation_sprite = None,
                    ToGuiMessage::RequestQuit => {
                        self.tx.send(FromGuiMessage::Quit)?;
                        break 'running;
                    }
                    ToGuiMessage::UpdateGrid => {
                        grid_sprites.clear();
                        for x in 0..nx {
                            for y in 0..ny {
                                let mut sprite = self.assets.background.clone();
                                sprite.index = Some((x, y));
                                sprite.rect = Some(Rect::new(
                                    (x_min + s * x) as i32,
                                    (y_min + s * y) as i32,
                                    s,
                                    s,
                                ));
                                grid_sprites.push(sprite);
                            }
                        }
                    }
                    ToGuiMessage::UpdateBuildings => {
                        if let Ok(game) = self.game.lock() {
                            building_sprites = (&game)
                                .buildings
                                .iter()
                                .map(|(pos, building)| {
                                    let bs = self.assets.buildings[&building].size;
                                    Sprite {
                                        size: bs,
                                        index: Some(*pos),
                                        path: self.assets.buildings[&building].path.clone(),
                                        building: Some(building.clone()),
                                        rect: Some(Rect::new(
                                            (x_min + s * (pos.0 - bs)) as i32,
                                            (y_min + s * (pos.1 - bs)) as i32,
                                            s * (1 + 2 * bs),
                                            s * (1 + 2 * bs),
                                        )),
                                    }
                                })
                                .collect::<Vec<_>>()
                                .clone();
                        }
                    }
                    ToGuiMessage::Quit => break 'running,
                }
            }
        }
        Ok(())
    }
}
