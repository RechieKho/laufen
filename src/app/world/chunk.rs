use super::block;
use super::morton;
use super::morton::Morton;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkKey(glam::UVec3);

impl From<glam::UVec3> for ChunkKey {
    fn from(p_value: glam::UVec3) -> Self {
        ChunkKey(p_value)
    }
}

impl From<ChunkKey> for glam::UVec3 {
    fn from(p_value: ChunkKey) -> Self {
        p_value.0
    }
}

impl std::ops::Deref for ChunkKey {
    type Target = glam::UVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialOrd for ChunkKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_length_squared = self.length_squared();
        let other_length_squared = other.length_squared();
        if self_length_squared < other_length_squared {
            return std::cmp::Ordering::Less;
        }
        if self_length_squared > other_length_squared {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Equal
    }
}

pub trait ChunkLoader<T: block::Block> {
    fn load_chunk(p_key: ChunkKey) -> Chunk<T>;
}

pub trait ChunkSaver<T: block::Block> {
    fn save_chunk(p_key: ChunkKey, p_chunk: &Chunk<T>);
}

pub type ChunkMorton = morton::Morton4;

pub struct Chunk<T: block::Block> {
    blocks: [T; ChunkMorton::MAX as _],
}

impl<T: block::Block> Default for Chunk<T> {
    fn default() -> Self {
        Self {
            blocks: [T::default(); ChunkMorton::MAX as _],
        }
    }
}

impl<T: block::Block> Chunk<T> {
    pub fn get(&self, p_code: u16) -> &T {
        &self.blocks[p_code as usize]
    }

    pub fn get_mut(&mut self, p_code: u16) -> &mut T {
        &mut self.blocks[p_code as usize]
    }

    pub fn get_from_position(&self, p_position: glam::UVec3) -> &T {
        let (code, _) = ChunkMorton::consume(p_position);
        &self.blocks[code as usize]
    }

    pub fn get_from_position_mut(&mut self, p_position: glam::UVec3) -> &mut T {
        let (code, _) = ChunkMorton::consume(p_position);
        &mut self.blocks[code as usize]
    }
}

pub type ChunkMap<T> = std::collections::BTreeMap<ChunkKey, Chunk<T>>;

pub struct SampleChunkLoader();

impl<T: block::Block> ChunkLoader<T> for SampleChunkLoader {
    fn load_chunk(_p_key: ChunkKey) -> Chunk<T> {
        // Just spawn one block.
        let mut chunk = Chunk::<T>::default();
        *chunk.get_from_position_mut(glam::UVec3::ZERO) = T::default_filled();
        chunk
    }
}

pub struct SampleChunkSaver();

impl<T: block::Block> ChunkSaver<T> for SampleChunkSaver {
    fn save_chunk(_p_key: ChunkKey, _p_chunk: &Chunk<T>) {
        // Ignored...
    }
}
