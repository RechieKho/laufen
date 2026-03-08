use super::cube;
use super::morton;
use super::morton::Morton;
use super::slot;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChunkKey(glam::IVec3);

impl From<glam::IVec3> for ChunkKey {
    fn from(p_value: glam::IVec3) -> Self {
        ChunkKey(p_value)
    }
}

impl From<ChunkKey> for glam::IVec3 {
    fn from(p_value: ChunkKey) -> Self {
        p_value.0
    }
}

impl std::ops::Deref for ChunkKey {
    type Target = glam::IVec3;

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
        let x_diff = self.x - other.x;
        let y_diff = self.y - other.y;
        let z_diff = self.z - other.z;

        if x_diff != 0 {
            return if x_diff < 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }

        if y_diff != 0 {
            return if y_diff < 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }

        if z_diff != 0 {
            return if z_diff < 0 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }

        std::cmp::Ordering::Equal
    }
}

pub struct ChunkLoadData {
    pub chunk: Chunk,
    pub require_hidden_face_baking: bool,
}

pub trait ChunkLoader: Send + Sync {
    fn load_chunk(&mut self, p_key: ChunkKey) -> anyhow::Result<ChunkLoadData>;
}

pub trait ChunkSaver: Send + Sync {
    fn save_chunk(&mut self, p_key: ChunkKey, p_chunk: &Chunk) -> anyhow::Result<()>;
}

pub type ChunkMorton = morton::Morton4;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Chunk {
    #[serde(with = "serde_arrays")]
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

    pub fn get_from_position(&self, p_position: glam::IVec3) -> &slot::Slot {
        let (code, _) = ChunkMorton::consume(p_position);
        &self.slots[code as usize]
    }

    pub fn get_from_position_mut(&mut self, p_position: glam::IVec3) -> &mut slot::Slot {
        let (code, _) = ChunkMorton::consume(p_position);
        &mut self.slots[code as usize]
    }
}

pub type ChunkMap = rapidhash::RapidHashMap<ChunkKey, Chunk>;

#[derive(Default)]
pub struct SampleChunkLoader();

impl ChunkLoader for SampleChunkLoader {
    fn load_chunk(&mut self, _p_key: ChunkKey) -> anyhow::Result<ChunkLoadData> {
        let mut chunk = Chunk::default();
        *chunk.get_from_position_mut(glam::IVec3::ZERO) = slot::Slot {
            cube_instance: Some(cube::CubeInstance::default()),
            ..Default::default()
        };
        *chunk.get_from_position_mut(glam::IVec3::new(1, 0, 1)) = slot::Slot {
            cube_instance: Some(cube::CubeInstance::default()),
            ..Default::default()
        };
        *chunk.get_from_position_mut(glam::IVec3::new(1, 0, 0)) = slot::Slot {
            cube_instance: Some(cube::CubeInstance::default()),
            ..Default::default()
        };
        *chunk.get_from_position_mut(glam::IVec3::new(0, 0, 1)) = slot::Slot {
            cube_instance: Some(cube::CubeInstance::default()),
            ..Default::default()
        };
        *chunk.get_from_position_mut(glam::IVec3::new(2, 0, 1)) = slot::Slot {
            cube_instance: Some(cube::CubeInstance::default()),
            ..Default::default()
        };
        Ok(ChunkLoadData {
            chunk,
            require_hidden_face_baking: true,
        })
    }
}

#[derive(Default)]
pub struct SampleChunkSaver();

impl ChunkSaver for SampleChunkSaver {
    fn save_chunk(&mut self, _p_key: ChunkKey, _p_chunk: &Chunk) -> anyhow::Result<()> {
        Ok(())
    }
}
