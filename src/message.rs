use super::game::Building;

#[derive(Debug)]
pub enum FromGuiMessage {
    Build((u32, u32), Building),
}
