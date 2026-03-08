use crate::adapter::shared;

pub mod active_geosphere;
pub mod chunk;
pub mod cube;
pub mod morton;
pub mod passive_geosphere;
pub mod point_cluster;
pub mod slot;
pub mod spectator;

pub trait Geosphere {
    fn slot(&mut self, p_position: glam::IVec3) -> &mut slot::Slot;
    fn slot_cube(&mut self, p_position: glam::IVec3) -> (&mut slot::Slot, Option<cube::Cube>);
}

pub type SharedGeosphere = shared::Shared<dyn Geosphere + 'static + Send + Sync>;
