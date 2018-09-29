use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    pub player: Option<usize>,
    pub size: (u32, u32),
    pub scores: Vec<ScoreEntry>,
    pub buildings: HashMap<(u32, u32), Building>,
    pub rejoin: String,
}

#[derive(Debug, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: usize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            player: None,
            size: (5, 5),
            scores: Vec::new(),
            buildings: HashMap::new(),
            rejoin: String::new(),
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
