#[derive(shipyard::Component, Default, Clone)]
pub struct Spatial {
    pub position: glam::Vec3,
    pub rotation: glam::EulerRot,
}
