use super::cube;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Slot {
    pub cube_instance: Option<cube::CubeInstance>,

    pub display_upward_quad: bool,
    pub display_downward_quad: bool,
    pub display_left_quad: bool,
    pub display_right_quad: bool,
    pub display_front_quad: bool,
    pub display_back_quad: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            cube_instance: None,
            display_upward_quad: true,
            display_downward_quad: true,
            display_left_quad: true,
            display_right_quad: true,
            display_front_quad: true,
            display_back_quad: true,
        }
    }
}
