use super::morton;
use super::morton::Morton;
use super::slot;

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

pub trait ChunkLoader {
    fn load_chunk(p_key: ChunkKey) -> Chunk;
}

pub trait ChunkSaver {
    fn save_chunk(p_key: ChunkKey, p_chunk: &Chunk);
}

pub type ChunkMorton = morton::Morton4;

pub struct Chunk {
    slots: [slot::Slot; ChunkMorton::MAX as _],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            slots: [slot::Slot::default(); ChunkMorton::MAX as _],
        }
    }
}

impl Chunk {
    pub fn get(&self, p_code: u16) -> &slot::Slot {
        &self.slots[p_code as usize]
    }

    pub fn get_mut(&mut self, p_code: u16) -> &mut slot::Slot {
        &mut self.slots[p_code as usize]
    }

    pub fn get_from_position(&self, p_position: glam::UVec3) -> &slot::Slot {
        let (code, _) = ChunkMorton::consume(p_position);
        &self.slots[code as usize]
    }

    pub fn get_from_position_mut(&mut self, p_position: glam::UVec3) -> &mut slot::Slot {
        let (code, _) = ChunkMorton::consume(p_position);
        &mut self.slots[code as usize]
    }
}

pub type ChunkMap = std::collections::BTreeMap<ChunkKey, Chunk>;

pub struct SampleChunkLoader();

impl ChunkLoader for SampleChunkLoader {
    fn load_chunk(_p_key: ChunkKey) -> Chunk {
        Chunk::default()
    }
}

pub struct SampleChunkSaver();

impl ChunkSaver for SampleChunkSaver {
    fn save_chunk(_p_key: ChunkKey, _p_chunk: &Chunk) {
        // Ignored...
    }
}
