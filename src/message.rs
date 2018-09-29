use super::game::Building;

#[derive(Debug)]
pub enum FromGuiMessage {
    Build((u32, u32), Building),
    Excavate((u32, u32)),
    Quit,
}

#[derive(Debug)]
pub enum ToGuiMessage {
    Message(String, String),
    ExcavateResult(i32, Building, (u32, u32)),
}
