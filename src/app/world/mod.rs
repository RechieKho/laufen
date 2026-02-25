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

    fn load(&mut self, p_key: chunk::ChunkKey) {
        self.chunk_map
            .insert(p_key, self.loader.load_chunk(p_key).unwrap());
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

    pub fn get_slot(&mut self, p_position: glam::IVec3) -> &slot::Slot {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load(key);
        }
        let chunk = self.chunk_map.get(&key).unwrap();
        chunk.get(code)
    }

    pub fn get_slot_mut<L: chunk::ChunkLoader, S: chunk::ChunkSaver>(
        &mut self,
        p_position: glam::IVec3,
    ) -> &mut slot::Slot {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load(key);
        }
        let chunk = self.chunk_map.get_mut(&key).unwrap();
        chunk.get_mut(code)
    }
}
