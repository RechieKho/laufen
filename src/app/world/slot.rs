use super::cube;

#[derive(Default, Clone, Copy)]
pub struct Slot {
    pub cube_instance: Option<cube::CubeInstance>,
}
