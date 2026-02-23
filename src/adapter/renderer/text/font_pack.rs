pub use gfx_glyph::ab_glyph::Font;
pub use gfx_glyph::ab_glyph::FontArc;

pub struct FontPack<T: Font> {
    pub normal: T,
    pub bold: Option<T>,
    pub italic: Option<T>,
    pub bold_italic: Option<T>,
}

impl FontPack<FontArc> {
    pub fn try_load_default() -> anyhow::Result<Self> {
        let normal = FontArc::try_from_slice(include_bytes!("./monocraft/Monocraft.ttf"))?;
        let bold = FontArc::try_from_slice(include_bytes!("./monocraft/Monocraft-Bold.ttf"))?;
        let italic = FontArc::try_from_slice(include_bytes!("./monocraft/Monocraft-Italic.ttf"))?;
        let bold_italic =
            FontArc::try_from_slice(include_bytes!("./monocraft/Monocraft-Bold-Italic.ttf"))?;

        Ok(Self {
            normal,
            bold: Some(bold),
            italic: Some(italic),
            bold_italic: Some(bold_italic),
        })
    }
}
