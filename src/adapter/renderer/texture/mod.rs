use image::GenericImageView;

use super::bind_group;
use super::bind_group::ToBindGroupContext;
use super::rendering_server;

#[derive(Clone, partially::Partial)]
#[partially(derive(Default))]
pub struct TextureContextBuilder<'a> {
    pub label: Option<&'a str>,

    pub texture_mip_level_count: u32,
    pub texture_sample_count: u32,
    pub texture_dimension: rendering_server::TextureDimension,
    pub texture_format: rendering_server::TextureFormat,
    pub texture_usage: rendering_server::TextureUsages,
    pub texture_view_formats: &'a [rendering_server::TextureFormat],

    pub sampler_address_mode_u: rendering_server::AddressMode,
    pub sampler_address_mode_v: rendering_server::AddressMode,
    pub sampler_address_mode_w: rendering_server::AddressMode,
    pub sampler_mag_filter: rendering_server::FilterMode,
    pub sampler_min_filter: rendering_server::FilterMode,
    pub sampler_mipmap_filter: rendering_server::MipmapFilterMode,
    pub sampler_compare: Option<rendering_server::CompareFunction>,
    pub sampler_lod_min_clamp: f32,
    pub sampler_lod_max_clamp: f32,
}

impl<'a> Default for TextureContextBuilder<'a> {
    fn default() -> Self {
        Self {
            label: None,

            texture_mip_level_count: 1,
            texture_sample_count: 1,
            texture_dimension: rendering_server::TextureDimension::D2,
            texture_format: rendering_server::TextureFormat::Rgba8UnormSrgb,
            texture_usage: rendering_server::TextureUsages::TEXTURE_BINDING
                | rendering_server::TextureUsages::COPY_DST,
            texture_view_formats: &[],

            sampler_address_mode_u: rendering_server::AddressMode::ClampToEdge,
            sampler_address_mode_v: rendering_server::AddressMode::ClampToEdge,
            sampler_address_mode_w: rendering_server::AddressMode::ClampToEdge,
            sampler_mag_filter: rendering_server::FilterMode::Nearest,
            sampler_min_filter: rendering_server::FilterMode::Nearest,
            sampler_mipmap_filter: rendering_server::MipmapFilterMode::Nearest,
            sampler_compare: None,
            sampler_lod_min_clamp: 0.0,
            sampler_lod_max_clamp: 32.0,
        }
    }
}

pub struct TextureContextBuilderParameters<'a> {
    pub server: &'a rendering_server::RenderingServer,
    pub texture_width: std::num::NonZeroU32,
    pub texture_height: std::num::NonZeroU32,
}

pub struct TextureContextBuilderParametersFromImage<'a> {
    pub server: &'a rendering_server::RenderingServer,
    pub image: &'a image::DynamicImage,
}

pub struct TextureContextBuilderParametersFromImageBytes<'a> {
    pub server: &'a rendering_server::RenderingServer,
    pub image_bytes: &'a [u8],
}

impl<'a> TextureContextBuilder<'a> {
    pub const DEFAULT_DEPTH_TEXTURE_FORMAT: rendering_server::TextureFormat =
        rendering_server::TextureFormat::Depth32Float;

    pub fn spawn_depth_texture(p_server: &rendering_server::RenderingServer) -> TextureContext {
        let builder = Self::as_depth_texture_builder();
        builder.build(TextureContextBuilderParameters {
            server: p_server,
            texture_width: std::num::NonZeroU32::try_from(
                p_server.surface_configuration().width.max(1),
            )
            .unwrap(),
            texture_height: std::num::NonZeroU32::try_from(
                p_server.surface_configuration().height.max(1),
            )
            .unwrap(),
        })
    }

    pub fn as_depth_texture_builder() -> Self {
        Self {
            label: Some("Depth texture"),

            texture_mip_level_count: 1,
            texture_sample_count: 1,
            texture_dimension: rendering_server::TextureDimension::D2,
            texture_format: Self::DEFAULT_DEPTH_TEXTURE_FORMAT,
            texture_usage: rendering_server::TextureUsages::RENDER_ATTACHMENT
                | rendering_server::TextureUsages::TEXTURE_BINDING,
            texture_view_formats: &[],

            sampler_address_mode_u: rendering_server::AddressMode::ClampToEdge,
            sampler_address_mode_v: rendering_server::AddressMode::ClampToEdge,
            sampler_address_mode_w: rendering_server::AddressMode::ClampToEdge,
            sampler_mag_filter: rendering_server::FilterMode::Linear,
            sampler_min_filter: rendering_server::FilterMode::Linear,
            sampler_mipmap_filter: rendering_server::MipmapFilterMode::Nearest,
            sampler_compare: Some(rendering_server::CompareFunction::LessEqual),
            sampler_lod_min_clamp: 0.0,
            sampler_lod_max_clamp: 100.0,
        }
    }

    pub fn build_from_sample_image(
        self,
        p_server: &rendering_server::RenderingServer,
    ) -> TextureContext {
        let sample_image_bytes = include_bytes!("./sample_image.png");
        self.build_from_image_bytes(TextureContextBuilderParametersFromImageBytes {
            server: p_server,
            image_bytes: sample_image_bytes,
        })
        .expect("Sample image should load successfully.")
    }

    pub fn build_from_image_bytes(
        self,
        p_parameters: TextureContextBuilderParametersFromImageBytes<'a>,
    ) -> anyhow::Result<TextureContext> {
        let image = image::load_from_memory(p_parameters.image_bytes)?;
        Ok(
            self.build_from_image(TextureContextBuilderParametersFromImage {
                server: p_parameters.server,
                image: &image,
            }),
        )
    }

    pub fn build_from_image(
        self,
        p_parameters: TextureContextBuilderParametersFromImage<'a>,
    ) -> TextureContext {
        let (texture_width, texture_height) = p_parameters.image.dimensions();
        let context = self.build(TextureContextBuilderParameters {
            server: p_parameters.server,
            texture_width: std::num::NonZeroU32::try_from(texture_width).unwrap(),
            texture_height: std::num::NonZeroU32::try_from(texture_height).unwrap(),
        });
        context
            .update_from_image(p_parameters.server, p_parameters.image)
            .expect("Built texture context should have texture size equal to image's dimension.");
        context
    }

    pub fn build(self, p_parameters: TextureContextBuilderParameters<'a>) -> TextureContext {
        let size = rendering_server::Extent3d {
            width: p_parameters.texture_width.get(),
            height: p_parameters.texture_height.get(),
            depth_or_array_layers: 1,
        };

        let texture =
            p_parameters
                .server
                .device()
                .create_texture(&rendering_server::TextureDescriptor {
                    label: self.label,
                    size,
                    mip_level_count: self.texture_mip_level_count,
                    sample_count: self.texture_sample_count,
                    dimension: self.texture_dimension,
                    format: self.texture_format,
                    usage: self.texture_usage,
                    view_formats: self.texture_view_formats,
                });
        let view = texture.create_view(&Default::default());
        let sampler =
            p_parameters
                .server
                .device()
                .create_sampler(&rendering_server::SamplerDescriptor {
                    address_mode_u: self.sampler_address_mode_u,
                    address_mode_v: self.sampler_address_mode_v,
                    address_mode_w: self.sampler_address_mode_w,
                    mag_filter: self.sampler_mag_filter,
                    min_filter: self.sampler_min_filter,
                    mipmap_filter: self.sampler_mipmap_filter,
                    compare: self.sampler_compare,
                    lod_min_clamp: self.sampler_lod_min_clamp,
                    lod_max_clamp: self.sampler_lod_max_clamp,
                    label: self.label,
                    ..Default::default()
                });

        TextureContext {
            texture,
            view,
            sampler,
        }
    }
}

#[derive(getset::Getters)]
pub struct TextureContext {
    #[getset(get = "pub")]
    pub(super) texture: rendering_server::Texture,
    #[getset(get = "pub")]
    pub(super) view: rendering_server::TextureView,
    #[getset(get = "pub")]
    pub(super) sampler: rendering_server::Sampler,
}

impl bind_group::ToBindGroupContext for TextureContext {
    const BIND_GROUP_LAYOUT_DESCRIPTOR: rendering_server::BindGroupLayoutDescriptor<'static> =
        rendering_server::BindGroupLayoutDescriptor {
            entries: &[
                rendering_server::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: rendering_server::ShaderStages::FRAGMENT,
                    ty: rendering_server::BindingType::Texture {
                        multisampled: false,
                        view_dimension: rendering_server::TextureViewDimension::D2,
                        sample_type: rendering_server::TextureSampleType::Float {
                            filterable: true,
                        },
                    },
                    count: None,
                },
                rendering_server::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: rendering_server::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: rendering_server::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Texture bind group layout"),
        };

    fn to_bind_group_context(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_label: Option<&str>,
    ) -> bind_group::BindGroupContext {
        let layout = self.create_bind_group_layout(p_server);
        let bind_group =
            p_server
                .device()
                .create_bind_group(&rendering_server::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        rendering_server::BindGroupEntry {
                            binding: 0,
                            resource: rendering_server::BindingResource::TextureView(&self.view),
                        },
                        rendering_server::BindGroupEntry {
                            binding: 1,
                            resource: rendering_server::BindingResource::Sampler(&self.sampler),
                        },
                    ],
                    label: p_label,
                });
        bind_group::BindGroupContext {
            bind_group_layout: layout,
            bind_group,
            bind_group_offset: 0,
        }
    }
}

impl TextureContext {
    pub fn update_from_image(
        &self,
        p_server: &rendering_server::RenderingServer,
        p_image: &image::DynamicImage,
    ) -> anyhow::Result<()> {
        let texture_size = self.texture.size();
        let image_size = p_image.dimensions();

        if texture_size.width != image_size.0 || texture_size.height != image_size.1 {
            return Err(anyhow::anyhow!(
                "The texture dimension and image dimension does not match."
            ));
        }

        let rgba = p_image.to_rgba8();

        p_server.queue().write_texture(
            rendering_server::TexelCopyTextureInfo {
                aspect: rendering_server::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: rendering_server::Origin3d::ZERO,
            },
            &rgba,
            rendering_server::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_size.0),
                rows_per_image: Some(image_size.1),
            },
            texture_size,
        );

        Ok(())
    }
}

#[derive(getset::Getters)]
pub struct BoundedTextureContext {
    #[getset(get = "pub")]
    texture_context: TextureContext,
    #[getset(get = "pub")]
    bind_group_context: bind_group::BindGroupContext,
}

impl BoundedTextureContext {
    pub fn new(
        p_server: &rendering_server::RenderingServer,
        p_texture_context: TextureContext,
    ) -> Self {
        let bind_group_context =
            p_texture_context.to_bind_group_context(p_server, Some("Bounded texture"));
        Self {
            texture_context: p_texture_context,
            bind_group_context,
        }
    }
}
