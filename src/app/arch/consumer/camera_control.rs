pub struct CameraControl {
    pub direction: glam::Vec3,
    pub view: CameraControlPersonView,
}

#[derive(Default)]
pub enum CameraControlPersonView {
    #[default]
    FirstPerson,
    SecondPerson {
        max_extension: f32,
    },
    ThirdPerson {
        max_extension: f32,
    },
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            direction: glam::Vec3::NEG_Z,
            view: CameraControlPersonView::FirstPerson,
        }
    }
}
