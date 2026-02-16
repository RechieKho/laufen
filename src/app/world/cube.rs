use stringid::create_hash_map;
use stringid::macros::strlookup_hashmap;
use stringid::macros::StringIdImpl;
use stringid::BufferStrStore;
use stringid::StringId;

#[strlookup_hashmap(key = u64, store = BufferStrStore, size = 128)]
struct CubeResourceIdLookUp;
#[derive(Debug, Clone, Copy, StringIdImpl)]
pub struct CubeResourceId(StringId<u64, CubeResourceIdLookUp>);

pub struct Cube {
    pub world_texture_image: image::DynamicImage,
}

type CubeMap = std::collections::BTreeMap<CubeResourceId, Cube>;

#[derive(Default)]
pub struct CubeRegistry {
    cubes: CubeMap,
}

impl CubeRegistry {
    pub fn register(&mut self, p_id: CubeResourceId, p_entry: Cube) {
        self.cubes.insert(p_id, p_entry);
    }
}
