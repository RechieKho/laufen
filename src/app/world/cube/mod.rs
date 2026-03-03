use std::str::FromStr;
use stringid::create_hash_map;
use stringid::macros::strlookup_hashmap;
use stringid::macros::StringIdImpl;
use stringid::BufferStrStore;
use stringid::StringId;

#[strlookup_hashmap(key = u64, store = BufferStrStore, size = 128)]
struct CubeResourceIdLookUp;
#[derive(Debug, Clone, Copy, StringIdImpl)]
pub struct CubeResourceId(StringId<u64, CubeResourceIdLookUp>);

impl CubeResourceId {
    pub fn new(p_name: &str) -> anyhow::Result<Self> {
        CubeResourceId::from_str(p_name)
            .map_err(|_err| anyhow::anyhow!("Unable to create cube resource id."))
    }

    pub fn try_default() -> anyhow::Result<Self> {
        Self::new("default")
    }
}

#[derive(Clone)]
pub struct Cube {
    pub world_texture_atlas_index_top: u32,
    pub world_texture_atlas_index_bottom: u32,
    pub world_texture_atlas_index_left: u32,
    pub world_texture_atlas_index_right: u32,
    pub world_texture_atlas_index_front: u32,
    pub world_texture_atlas_index_back: u32,
}

type CubeMap = std::collections::BTreeMap<CubeResourceId, Cube>;

#[derive(Default, getset::Getters)]
pub struct CubeRegistry {
    #[getset(get = "pub")]
    cubes: CubeMap,

    #[getset(get = "pub")]
    world_texture_images: Vec<image::DynamicImage>,
}

impl CubeRegistry {
    pub fn register(&mut self, p_id: CubeResourceId, p_entry: Cube) {
        self.cubes.insert(p_id, p_entry);
    }
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct CubeBuilder {
    pub id: CubeResourceId,
    pub world_texture_image_top: image::DynamicImage,
    pub world_texture_image_bottom: image::DynamicImage,
    pub world_texture_image_left: image::DynamicImage,
    pub world_texture_image_right: image::DynamicImage,
    pub world_texture_image_front: image::DynamicImage,
    pub world_texture_image_back: image::DynamicImage,
}

#[derive(Default)]
pub struct CubeRegistryBuilder {
    pub cube_builders: Vec<CubeBuilder>,
}

impl CubeRegistryBuilder {
    pub fn with_builtin() -> anyhow::Result<Self> {
        let dirt_top_data = include_bytes!("./dirt1.png");
        let dirt_side_data = include_bytes!("./dirt2.png");
        let dirt_bottom_data = include_bytes!("./dirt3.png");

        let dirt_top = image::load_from_memory(dirt_top_data)?;
        let dirt_side = image::load_from_memory(dirt_side_data)?;
        let dirt_bottom = image::load_from_memory(dirt_bottom_data)?;

        let mut builder = Self::default();
        builder.cube_builders.push(CubeBuilder {
            id: CubeResourceId::try_default()?,
            world_texture_image_top: dirt_top,
            world_texture_image_left: dirt_side.clone(),
            world_texture_image_right: dirt_side.clone(),
            world_texture_image_front: dirt_side.clone(),
            world_texture_image_back: dirt_side,
            world_texture_image_bottom: dirt_bottom,
        });
        Ok(builder)
    }

    pub fn build(self) -> CubeRegistry {
        let mut registry = CubeRegistry::default();
        let mut images = Vec::<image::DynamicImage>::default();

        for builder in self.cube_builders {
            let id = builder.id;
            let world_texture_atlas_index_top = images.len() as u32;
            images.push(builder.world_texture_image_top);
            let world_texture_atlas_index_bottom = images.len() as u32;
            images.push(builder.world_texture_image_bottom);
            let world_texture_atlas_index_left = images.len() as u32;
            images.push(builder.world_texture_image_left);
            let world_texture_atlas_index_right = images.len() as u32;
            images.push(builder.world_texture_image_right);
            let world_texture_atlas_index_front = images.len() as u32;
            images.push(builder.world_texture_image_front);
            let world_texture_atlas_index_back = images.len() as u32;
            images.push(builder.world_texture_image_back);

            let cube = Cube {
                world_texture_atlas_index_top,
                world_texture_atlas_index_bottom,
                world_texture_atlas_index_left,
                world_texture_atlas_index_right,
                world_texture_atlas_index_front,
                world_texture_atlas_index_back,
            };
            registry.register(id, cube);
        }

        registry.world_texture_images = images;
        registry
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum CubeInstanceOrientation {
    #[default]
    FORWARD,
    BACKWARD,
    UPWARD,
    DOWNWARD,
    LEFT,
    RIGHT,
}

impl From<CubeInstanceOrientation> for glam::Mat4 {
    fn from(p_value: CubeInstanceOrientation) -> Self {
        match p_value {
            CubeInstanceOrientation::FORWARD => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(0.0, 0.0, -1.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
            CubeInstanceOrientation::BACKWARD => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(0.0, 0.0, 1.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
            CubeInstanceOrientation::UPWARD => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(0.0, 1.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
            CubeInstanceOrientation::DOWNWARD => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(0.0, -1.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
            CubeInstanceOrientation::LEFT => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(-1.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
            CubeInstanceOrientation::RIGHT => glam::Mat4::look_to_rh(
                glam::Vec3::ZERO,
                glam::Vec3::new(1.0, 0.0, 0.0),
                glam::Vec3::new(0.0, 1.0, 0.0),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CubeInstance {
    pub id: CubeResourceId,
    pub orientation: CubeInstanceOrientation,
}

impl CubeInstance {
    pub fn try_default() -> anyhow::Result<Self> {
        Ok(Self {
            id: CubeResourceId::try_default()?,
            orientation: CubeInstanceOrientation::default(),
        })
    }
}
