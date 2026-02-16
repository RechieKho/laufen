use morton::Morton;

pub mod block;
pub mod chunk;
pub mod morton;

pub struct World<T: block::Block> {
    chunk_map: chunk::ChunkMap<T>,
}

impl<T: block::Block> World<T> {
    const MAX_CHUNK_CHEBYSHEV_DISTANCE: u32 = 7;

    fn load<L, S>(&mut self, p_key: chunk::ChunkKey)
    where
        L: chunk::ChunkLoader<T>,
        S: chunk::ChunkSaver<T>,
    {
        self.chunk_map.insert(p_key, L::load_chunk(p_key));

        let interest_chunk_position: glam::UVec3 = p_key.into();
        self.chunk_map.retain(|p_iter_key, p_iter_chunk| {
            let current_chunk_position: glam::UVec3 = (*p_iter_key).into();
            if interest_chunk_position.chebyshev_distance(current_chunk_position)
                < Self::MAX_CHUNK_CHEBYSHEV_DISTANCE
            {
                return true;
            }
            S::save_chunk(*p_iter_key, p_iter_chunk);
            false
        });
    }

    pub fn get_block<L, S>(&mut self, p_position: glam::UVec3) -> &T
    where
        L: chunk::ChunkLoader<T>,
        S: chunk::ChunkSaver<T>,
    {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load::<L, S>(key);
        }
        let chunk = self.chunk_map.get(&key).unwrap();
        chunk.get(code)
    }

    pub fn get_block_mut<L, S>(&mut self, p_position: glam::UVec3) -> &mut T
    where
        L: chunk::ChunkLoader<T>,
        S: chunk::ChunkSaver<T>,
    {
        let (code, remained) = chunk::ChunkMorton::consume(p_position);
        let key = chunk::ChunkKey::from(remained);
        if !self.chunk_map.contains_key(&key) {
            self.load::<L, S>(key);
        }
        let chunk = self.chunk_map.get_mut(&key).unwrap();
        chunk.get_mut(code)
    }
}
