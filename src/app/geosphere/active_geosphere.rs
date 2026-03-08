use super::chunk;
use super::cube;
use super::morton::Morton;
use super::point_cluster;
use super::slot;
use super::Geosphere;
use crate::adapter::shared;

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct ActiveGeosphereBuilder {
    pub loader: Box<dyn chunk::ChunkLoader>,
    pub saver: Box<dyn chunk::ChunkSaver>,
    pub cube_registry_builder: cube::CubeRegistryBuilder,
}

impl ActiveGeosphereBuilder {
    pub fn try_with_sample() -> anyhow::Result<Self> {
        Ok(Self {
            loader: Box::new(chunk::SampleChunkLoader::default()),
            saver: Box::new(chunk::SampleChunkSaver::default()),
            cube_registry_builder: cube::CubeRegistryBuilder::try_with_sample()?,
        })
    }

    pub fn build(self) -> ActiveGeosphere {
        ActiveGeosphere {
            chunk_map: chunk::ChunkMap::default(),
            loader: self.loader,
            saver: self.saver,
            cube_registry: self.cube_registry_builder.build(),
        }
    }
}

pub type SharedActiveGeosphere = shared::Shared<ActiveGeosphere>;

impl From<ActiveGeosphere> for SharedActiveGeosphere {
    fn from(p_value: ActiveGeosphere) -> Self {
        shared::share(p_value)
    }
}

#[derive(getset::Getters)]
pub struct ActiveGeosphere {
    chunk_map: chunk::ChunkMap,
    loader: Box<dyn chunk::ChunkLoader>,
    saver: Box<dyn chunk::ChunkSaver>,

    #[getset(get = "pub")]
    cube_registry: cube::CubeRegistry,
}

impl super::Geosphere for ActiveGeosphere {
    fn slot(&mut self, p_position: glam::IVec3) -> &mut slot::Slot {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        let chunk = self.chunk(&key);
        let slot = chunk.get_mut(code);
        slot
    }

    fn slot_cube(&mut self, p_position: glam::IVec3) -> (&mut slot::Slot, Option<cube::Cube>) {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load(key);
        }
        let chunk = self.chunk_map.get_mut(&key).unwrap();
        let cube = chunk
            .get(code)
            .cube_instance
            .and_then(|cube_instance| self.cube_registry.cubes().get(&cube_instance.id).cloned());
        let slot = chunk.get_mut(code);
        (slot, cube)
    }
}

impl ActiveGeosphere {
    fn bake_hidden_face(&mut self, p_position: glam::IVec3) {
        self.slot(p_position).display_upward_quad = self
            .slot(p_position + glam::IVec3::Y)
            .cube_instance
            .is_none();
        self.slot(p_position).display_downward_quad = self
            .slot(p_position + glam::IVec3::NEG_Y)
            .cube_instance
            .is_none();
        self.slot(p_position).display_left_quad = self
            .slot(p_position + glam::IVec3::NEG_X)
            .cube_instance
            .is_none();
        self.slot(p_position).display_right_quad = self
            .slot(p_position + glam::IVec3::X)
            .cube_instance
            .is_none();
        self.slot(p_position).display_front_quad = self
            .slot(p_position + glam::IVec3::NEG_Z)
            .cube_instance
            .is_none();
        self.slot(p_position).display_back_quad = self
            .slot(p_position + glam::IVec3::Z)
            .cube_instance
            .is_none();
    }

    pub fn bake_multiple_hidden_face(&mut self, p_point_cluster: point_cluster::PointCluster) {
        for point in p_point_cluster.into_iter() {
            self.bake_hidden_face(point);
        }
    }

    fn load(&mut self, p_key: chunk::ChunkKey) {
        let data = self.loader.load_chunk(p_key).unwrap();
        self.chunk_map.insert(p_key, data.chunk);

        if !data.require_hidden_face_baking {
            return;
        }

        let chunk_index = *p_key;
        let mut point_cluster = chunk::ChunkMorton::compute_cluster(chunk_index);

        point_cluster.min.x += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::NEG_X))
        {
            -1
        } else {
            1
        };

        point_cluster.min.y += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::NEG_Y))
        {
            -1
        } else {
            1
        };

        point_cluster.min.z += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::NEG_Z))
        {
            -1
        } else {
            1
        };

        point_cluster.max.x += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::X))
        {
            1
        } else {
            -1
        };

        point_cluster.max.y += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::Y))
        {
            1
        } else {
            -1
        };

        point_cluster.max.z += if self
            .chunk_map
            .contains_key(&chunk::ChunkKey::from(chunk_index + glam::IVec3::Z))
        {
            1
        } else {
            -1
        };

        self.bake_multiple_hidden_face(point_cluster);
    }

    pub fn chunk(&mut self, p_key: &chunk::ChunkKey) -> &mut chunk::Chunk {
        if !self.chunk_map.contains_key(p_key) {
            self.load(*p_key);
        }
        self.chunk_map.get_mut(p_key).unwrap()
    }

    pub fn purge_beyond(
        &mut self,
        p_slot_position: glam::IVec3,
        p_max_chebyshev_slot_distance: u32,
    ) {
        self.chunk_map.retain(|p_iter_key, p_iter_chunk| {
            let current_chunk_position: glam::IVec3 = (*p_iter_key).into();
            let current_slot_position =
                current_chunk_position * chunk::ChunkMorton::MAX_PER_COMPONENT as i32;
            let distance = p_slot_position.chebyshev_distance(current_slot_position);
            if distance < p_max_chebyshev_slot_distance {
                return true;
            }
            let _ = self.saver.save_chunk(*p_iter_key, p_iter_chunk);
            false
        });
    }
}
