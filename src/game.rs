use std::collections::HashMap;

pub struct Game {
    pub buildings: HashMap<(usize, usize), Building>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            buildings: HashMap::new(),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Building {
    House,
    Villa,
    Tower,
}
