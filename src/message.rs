use super::game::Building;

#[derive(Debug)]
pub enum FromGuiMessage {
    Build((u32, u32), Building),
    Excavate((u32, u32)),
    Skip,
    Quit,
}

#[derive(Debug)]
pub enum ToGuiMessage {
    Start,
    Message(String, String),
    ExcavateResult(i32, Option<Building>, (u32, u32)),
    ClearExcavate,
    ClearBuilding,
    SetBuilding((u32, u32), Building),
    Quit,
}
