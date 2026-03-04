use crate::adapter::renderer;
use crate::app::viewport;

pub type CubeResourceId = ustr::Ustr;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Cube {
    pub geosphere_texture_atlas_index_top: u32,
    pub geosphere_texture_atlas_index_bottom: u32,
    pub geosphere_texture_atlas_index_left: u32,
    pub geosphere_texture_atlas_index_right: u32,
    pub geosphere_texture_atlas_index_front: u32,
    pub geosphere_texture_atlas_index_back: u32,
}

type CubeMap = rapidhash::RapidHashMap<CubeResourceId, Cube>;

#[derive(Default, getset::Getters, serde::Serialize, serde::Deserialize, Clone)]
pub struct CubeRegistry {
    #[getset(get = "pub")]
    cubes: CubeMap,

    #[getset(get = "pub")]
    geosphere_serializable_texture_image:
        Vec<viewport::serializable_texture_image::SerializableTextureImage>,
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
    pub geosphere_texture_image_top: renderer::texture::TextureImage,
    pub geosphere_texture_image_bottom: renderer::texture::TextureImage,
    pub geosphere_texture_image_left: renderer::texture::TextureImage,
    pub geosphere_texture_image_right: renderer::texture::TextureImage,
    pub geosphere_texture_image_front: renderer::texture::TextureImage,
    pub geosphere_texture_image_back: renderer::texture::TextureImage,
}

#[derive(Default)]
pub struct CubeRegistryBuilder {
    pub cube_builders: Vec<CubeBuilder>,
}

impl CubeRegistryBuilder {
    pub fn try_with_sample() -> anyhow::Result<Self> {
        let dirt_top_data = include_bytes!("./dirt1.png");
        let dirt_side_data = include_bytes!("./dirt2.png");
        let dirt_bottom_data = include_bytes!("./dirt3.png");

        let dirt_top = image::load_from_memory(dirt_top_data)?.to_rgba8();
        let dirt_side = image::load_from_memory(dirt_side_data)?.to_rgba8();
        let dirt_bottom = image::load_from_memory(dirt_bottom_data)?.to_rgba8();

        let mut builder = Self::default();
        builder.cube_builders.push(CubeBuilder {
            id: CubeResourceId::default(),
            geosphere_texture_image_top: dirt_top,
            geosphere_texture_image_left: dirt_side.clone(),
            geosphere_texture_image_right: dirt_side.clone(),
            geosphere_texture_image_front: dirt_side.clone(),
            geosphere_texture_image_back: dirt_side,
            geosphere_texture_image_bottom: dirt_bottom,
        });
        Ok(builder)
    }

    pub fn build(self) -> CubeRegistry {
        let mut registry = CubeRegistry::default();
        let mut images =
            Vec::<viewport::serializable_texture_image::SerializableTextureImage>::default();

        for builder in self.cube_builders {
            let id = builder.id;
            let geosphere_texture_atlas_index_top = images.len() as u32;
            images.push(builder.geosphere_texture_image_top.into());
            let geosphere_texture_atlas_index_bottom = images.len() as u32;
            images.push(builder.geosphere_texture_image_bottom.into());
            let geosphere_texture_atlas_index_left = images.len() as u32;
            images.push(builder.geosphere_texture_image_left.into());
            let geosphere_texture_atlas_index_right = images.len() as u32;
            images.push(builder.geosphere_texture_image_right.into());
            let geosphere_texture_atlas_index_front = images.len() as u32;
            images.push(builder.geosphere_texture_image_front.into());
            let geosphere_texture_atlas_index_back = images.len() as u32;
            images.push(builder.geosphere_texture_image_back.into());

            let cube = Cube {
                geosphere_texture_atlas_index_top,
                geosphere_texture_atlas_index_bottom,
                geosphere_texture_atlas_index_left,
                geosphere_texture_atlas_index_right,
                geosphere_texture_atlas_index_front,
                geosphere_texture_atlas_index_back,
            };
            registry.register(id, cube);
        }

        registry.geosphere_serializable_texture_image = images;
        registry
    }
}

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct CubeInstance {
    pub id: CubeResourceId,
    pub orientation: CubeInstanceOrientation,
}
