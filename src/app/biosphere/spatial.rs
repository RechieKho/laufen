#[derive(shipyard::Component, Default)]
pub struct Spatial {
    pub position: glam::Vec3,
    pub rotation: glam::EulerRot,
}
