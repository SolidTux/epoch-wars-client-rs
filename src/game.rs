use failure::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Game {
    pub player: Option<usize>,
    pub size: (u32, u32),
    pub scores: Vec<ScoreEntry>,
    pub buildings: HashMap<(u32, u32), Building>,
    pub prices: HashMap<Building, u32>,
    pub tower_count: u32,
    pub turn: u32,
    pub rejoin: String,
}

#[derive(Debug, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: isize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            player: None,
            size: (5, 5),
            scores: Vec::new(),
            buildings: HashMap::new(),
            turn: 0,
            rejoin: String::new(),
            tower_count: 0,
            prices: HashMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Building {
    House,
    Villa,
    Tower,
}

impl ::std::str::FromStr for Building {
    type Err = Error;
    fn from_str(s: &str) -> Result<Building, Error> {
        match s.to_lowercase().as_str() {
            "house" => Ok(Building::House),
            "villa" => Ok(Building::Villa),
            "tower" => Ok(Building::Tower),
            _ => Err(format_err!("Unknown building type.")),
        }
    }
}
