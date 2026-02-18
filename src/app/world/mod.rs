use morton::Morton;

pub mod chunk;
pub mod cube;
pub mod morton;
pub mod slot;
pub mod uaabb;

pub struct World {
    chunk_map: chunk::ChunkMap,
    loader: Box<dyn chunk::ChunkLoader>,
    saver: Box<dyn chunk::ChunkSaver>,
}

impl World {
    const MAX_CHUNK_CHEBYSHEV_DISTANCE: u32 = 7;

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

        let interest_chunk_position: glam::UVec3 = p_key.into();
        self.chunk_map.retain(|p_iter_key, p_iter_chunk| {
            let current_chunk_position: glam::UVec3 = (*p_iter_key).into();
            if interest_chunk_position.chebyshev_distance(current_chunk_position)
                < Self::MAX_CHUNK_CHEBYSHEV_DISTANCE
            {
                return true;
            }
            let _ = self.saver.save_chunk(*p_iter_key, p_iter_chunk);
            false
        });
    }

    pub fn get_slot(&mut self, p_position: glam::UVec3) -> &slot::Slot {
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
        p_position: glam::UVec3,
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
