use super::rendering_server;

pub use wgpu_text::glyph_brush::FontId;
pub use wgpu_text::glyph_brush::Section as TextSection;
pub use wgpu_text::glyph_brush::Text;
pub use wgpu_text::BrushError;
pub use wgpu_text::TextBrush;

pub mod font_pack;

#[derive(partially::Partial, Default)]
#[partially(derive(Default))]
pub struct TextBrushContextBuilder {
    pub depth_stencil_state: Option<rendering_server::DepthStencilState>,
}

pub struct TextBrushContextBuilderParameters<'a, T: font_pack::Font> {
    pub server: &'a rendering_server::RenderingServer,
    pub font_pack: font_pack::FontPack<T>,
}

impl TextBrushContextBuilder {
    pub fn build_with_default_font(
        self,
        p_server: &rendering_server::RenderingServer,
    ) -> anyhow::Result<TextBrushContext<font_pack::FontArc>> {
        let font_pack = font_pack::FontPack::try_load_default()?;
        self.build(TextBrushContextBuilderParameters {
            server: p_server,
            font_pack,
        })
    }

    pub fn build<'a, T: font_pack::Font>(
        self,
        p_parameters: TextBrushContextBuilderParameters<'a, T>,
    ) -> anyhow::Result<TextBrushContext<T>> {
        let mut builder = wgpu_text::BrushBuilder::using_font(p_parameters.font_pack.normal)
            .with_depth_stencil(self.depth_stencil_state)
            .initial_cache_size((512, 512));

        let normal = FontId::default();

        let italic = p_parameters
            .font_pack
            .italic
            .map(|p_font| builder.add_font(p_font));
        let bold = p_parameters
            .font_pack
            .bold
            .map(|p_font| builder.add_font(p_font));
        let bold_italic = p_parameters
            .font_pack
            .bold_italic
            .map(|p_font| builder.add_font(p_font));

        let brush = builder.build(
            p_parameters.server.device(),
            p_parameters.server.surface_configuration().width,
            p_parameters.server.surface_configuration().height,
            p_parameters.server.surface_configuration().format,
        );

        Ok(TextBrushContext {
            brush,
            normal,
            italic,
            bold,
            bold_italic,
        })
    }
}

pub struct TextBrushContext<T: font_pack::Font = font_pack::FontArc> {
    pub brush: TextBrush<T>,
    pub normal: FontId,
    pub bold: Option<FontId>,
    pub italic: Option<FontId>,
    pub bold_italic: Option<FontId>,
}

impl<T: font_pack::Font + Sync> TextBrushContext<T> {
    pub fn resize(&self, p_width: f32, p_height: f32, p_queue: &rendering_server::Queue) {
        self.brush.resize_view(p_width, p_height, p_queue);
    }

    pub fn queue<'a, S, I>(
        &mut self,
        p_server: &rendering_server::RenderingServer,
        p_sections: I,
    ) -> anyhow::Result<(), BrushError>
    where
        S: Into<std::borrow::Cow<'a, TextSection<'a>>>,
        I: IntoIterator<Item = S>,
    {
        self.brush
            .queue(p_server.device(), p_server.queue(), p_sections)
    }
}

impl<T: font_pack::Font + Sync> rendering_server::SubmitToRenderPass for TextBrushContext<T> {
    fn submit<'a>(&self, p_render_pass: &mut wgpu::RenderPass<'a>) {
        self.brush.draw(p_render_pass);
    }
}
