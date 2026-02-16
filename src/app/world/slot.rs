use super::cube;

#[derive(Default, Clone, Copy)]
pub struct Slot {
    pub cube_id: Option<cube::CubeResourceId>,
}
