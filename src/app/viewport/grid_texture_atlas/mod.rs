use repr_trait::C;

use crate::adapter::renderer::bind_group;
use crate::adapter::renderer::bind_group::ToBindGroupContext;
use crate::adapter::renderer::buffer;
use crate::adapter::renderer::buffer::ToBuffer;
use crate::adapter::renderer::rendering_server;
use crate::adapter::renderer::texture;
use crate::adapter::renderer::uniform_buffer;

#[derive(getset::Getters, getset::MutGetters)]
pub struct GridTextureDivisionContext {
    pub uniform_data: GridTextureDivisionUniform,

    #[getset(get = "pub", get_mut = "pub")]
    uniform_buffer: uniform_buffer::UniformBuffer,
    #[getset(get = "pub", get_mut = "pub")]
    bind_group_context: bind_group::BindGroupContext,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
pub struct GridTextureDivisionUniform {
    division: u32,
}

impl buffer::BufferElement for GridTextureDivisionUniform {}
impl uniform_buffer::UniformBufferElement for GridTextureDivisionUniform {}

impl GridTextureDivisionContext {
    pub fn new(
        p_server: &rendering_server::RenderingServer,
        p_uniform_data: GridTextureDivisionUniform,
    ) -> Self {
        let uniform_buffer = uniform_buffer::ToUniformBuffer(&[p_uniform_data])
            .to_buffer(p_server, Some("Grid texture division uniform buffer"));
        let bind_group_context = uniform_buffer
            .to_bind_group_context(p_server, Some("Grid texture division bind group"));

        Self {
            uniform_data: p_uniform_data,
            uniform_buffer,
            bind_group_context,
        }
    }
}

pub struct GridTextureAtlas {
    pub bounded_texture_context: texture::BoundedTextureContext,
    pub division_context: GridTextureDivisionContext,
}

#[derive(partially::Partial)]
#[partially(derive(Default))]
pub struct GridTextureAtlasBuilder<'a> {
    pub cell_size: std::num::NonZeroU32,
    pub images: &'a [image::DynamicImage],
    pub texture_label: Option<&'a str>,
}

pub struct GridTextureAtlasBuilderParameters<'a> {
    pub server: &'a rendering_server::RenderingServer,
}

impl<'a> Default for GridTextureAtlasBuilder<'a> {
    fn default() -> Self {
        Self {
            cell_size: std::num::NonZeroU32::new(24).unwrap(),
            images: &[],
            texture_label: Some("Grid texture atlas"),
        }
    }
}

impl<'a> GridTextureAtlasBuilder<'a> {
    pub fn get_sample_images() -> anyhow::Result<Vec<image::DynamicImage>> {
        let image_one_bytes = include_bytes!("sample_grid_cell_texture_one.png");
        let image_two_bytes = include_bytes!("sample_grid_cell_texture_two.png");
        Ok(vec![
            image::load_from_memory(image_one_bytes)?,
            image::load_from_memory(image_two_bytes)?,
        ])
    }

    pub fn build(
        self,
        p_parameters: GridTextureAtlasBuilderParameters<'a>,
    ) -> anyhow::Result<GridTextureAtlas> {
        if self.images.is_empty() {
            let image = image::DynamicImage::new(
                self.cell_size.get(),
                self.cell_size.get(),
                image::ColorType::Rgb8,
            );
            let texture_context = p_parameters
                .server
                .load_texture_from_image(&image, self.texture_label)?;
            let bounded_texture_context =
                texture::BoundedTextureContext::new(p_parameters.server, texture_context);
            return Ok(GridTextureAtlas {
                bounded_texture_context,
                division_context: GridTextureDivisionContext::new(
                    p_parameters.server,
                    GridTextureDivisionUniform { division: 0 },
                ),
            });
        }

        let input_image_count = self.images.len() as u32;
        let mut image_count_per_axis = input_image_count.isqrt();

        loop {
            let remainding_image_count =
                input_image_count as i32 - image_count_per_axis.pow(2) as i32;
            if remainding_image_count <= 0 {
                break;
            }
            image_count_per_axis += 1;
        }

        let mut image = image::DynamicImage::new(
            image_count_per_axis * self.cell_size.get(),
            image_count_per_axis * self.cell_size.get(),
            image::ColorType::Rgb8,
        );

        for (i, input_image) in self.images.iter().enumerate() {
            let resized = input_image.resize_exact(
                self.cell_size.get(),
                self.cell_size.get(),
                image::imageops::FilterType::Nearest,
            );
            let x = i % (image_count_per_axis as usize);
            let y = i / (image_count_per_axis as usize);
            image::imageops::overlay(
                &mut image,
                &resized,
                (x as u32 * self.cell_size.get()) as _,
                (y as u32 * self.cell_size.get()) as _,
            );
        }

        let texture_context = p_parameters
            .server
            .load_texture_from_image(&image, self.texture_label)?;
        let bounded_texture_context =
            texture::BoundedTextureContext::new(p_parameters.server, texture_context);
        Ok(GridTextureAtlas {
            bounded_texture_context,
            division_context: GridTextureDivisionContext::new(
                p_parameters.server,
                GridTextureDivisionUniform {
                    division: image_count_per_axis,
                },
            ),
        })
    }
}
