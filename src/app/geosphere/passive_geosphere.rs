use super::chunk;
use super::cube;
use super::morton::Morton;
use super::slot;
use crate::adapter::renderer;
use crate::adapter::shared;

pub type SharedPassiveGeosphere = shared::Shared<PassiveGeosphere>;

#[derive(Default)]
pub struct PassiveGeosphere {
    pub chunk_map: chunk::ChunkMap,
    pub cube_registry: cube::CubeRegistry,
}

impl From<PassiveGeosphere> for SharedPassiveGeosphere {
    fn from(p_value: PassiveGeosphere) -> Self {
        shared::share(p_value)
    }
}

impl super::Geosphere for PassiveGeosphere {
    fn slot(&mut self, p_position: glam::IVec3) -> &mut slot::Slot {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        self.chunk_map.entry(key).or_default().get_mut(code)
    }

    fn slot_cube(&mut self, p_position: glam::IVec3) -> (&mut slot::Slot, Option<cube::Cube>) {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        let chunk = self.chunk_map.entry(key).or_default();
        let slot = chunk.get_mut(code);
        let cube = slot
            .cube_instance
            .as_ref()
            .and_then(|cube_instance| self.cube_registry.cubes().get(&cube_instance.id).cloned());
        (slot, cube)
    }
}

impl PassiveGeosphere {
    pub fn deserialize_geosphere_texture_image(self) -> Vec<renderer::texture::TextureImage> {
        self.cube_registry
            .geosphere_serializable_texture_image()
            .iter()
            .map(|p_image| p_image.clone().to_texture_image().unwrap())
            .collect::<Vec<renderer::texture::TextureImage>>()
    }
}
