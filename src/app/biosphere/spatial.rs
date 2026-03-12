#[derive(shipyard::Component, Default, serde::Serialize, serde::Deserialize, Clone)]
pub struct Spatial {
    pub position: glam::Vec3,
    pub direction: glam::Vec3,
}
