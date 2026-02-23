use super::rendering_server;

pub use wgpu_text::glyph_brush::FontId;
pub use wgpu_text::glyph_brush::Section as TextSection;
pub use wgpu_text::glyph_brush::Text;
pub use wgpu_text::BrushError;
pub use wgpu_text::TextBrush;

pub mod font_pack;

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
