use crate::app::geosphere;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryQueryInput();

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistryQueryOutput {
    cube_registry: geosphere::cube::CubeRegistry,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum CubeRegistryQuery {
    Input(CubeRegistryQueryInput),
    Output(CubeRegistryQueryOutput),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GeosphereQueryInput {
    position: glam::IVec3,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GeosphereQueryOutput {
    slot: geosphere::slot::Slot,
    cube: Option<geosphere::cube::Cube>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum GeosphereQuery {
    Input(GeosphereQueryInput),
    Output(GeosphereQueryOutput),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Relay {
    CubeRegistry(CubeRegistryQuery),
    Geosphere(GeosphereQuery),
}
