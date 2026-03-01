use morton::Morton;

pub mod chunk;
pub mod cube;
pub mod morton;
pub mod point_cluster;
pub mod slot;

pub struct World {
    chunk_map: chunk::ChunkMap,
    loader: Box<dyn chunk::ChunkLoader>,
    saver: Box<dyn chunk::ChunkSaver>,
}

impl World {
    pub fn with_sample_loader_saver() -> Self {
        Self {
            chunk_map: chunk::ChunkMap::default(),
            loader: Box::new(chunk::SampleChunkLoader::default()),
            saver: Box::new(chunk::SampleChunkSaver::default()),
        }
    }

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

    pub fn purge_beyond(&mut self, p_center: glam::IVec3, p_max_chebyshev_distance: u32) {
        self.chunk_map.retain(|p_iter_key, p_iter_chunk| {
            let current_chunk_position: glam::IVec3 = (*p_iter_key).into();
            let distance = p_center.chebyshev_distance(current_chunk_position);
            if distance < p_max_chebyshev_distance {
                return true;
            }
            let _ = self.saver.save_chunk(*p_iter_key, p_iter_chunk);
            false
        });
    }

    pub fn slot(&mut self, p_position: glam::IVec3) -> &mut slot::Slot {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load(key);
        }
        let chunk = self.chunk_map.get_mut(&key).unwrap();
        chunk.get_mut(code)
    }
}
