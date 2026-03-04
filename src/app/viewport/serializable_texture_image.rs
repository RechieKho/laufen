use crate::adapter::renderer;

#[derive(getset::Getters, serde::Serialize, serde::Deserialize, Clone)]
pub struct SerializableTextureImage {
    #[getset(get = "pub")]
    width: u32,
    #[getset(get = "pub")]
    height: u32,
    #[getset(get = "pub")]
    data: renderer::texture::TextureImageContainer,
}

impl From<renderer::texture::TextureImage> for SerializableTextureImage {
    fn from(p_value: renderer::texture::TextureImage) -> Self {
        Self {
            width: p_value.width(),
            height: p_value.height(),
            data: p_value.into_raw(),
        }
    }
}

impl SerializableTextureImage {
    pub fn to_texture_image(self) -> Option<renderer::texture::TextureImage> {
        renderer::texture::TextureImage::from_raw(self.width, self.height, self.data)
    }
}
