use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    pub player: Option<usize>,
    pub size: (u32, u32),
    pub buildings: HashMap<(u32, u32), Building>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            player: None,
            size: (5, 5),
            buildings: HashMap::new(),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Building {
    House,
    Villa,
    Tower,
}
