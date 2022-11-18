use wgpu::{util::StagingBelt, CommandEncoder, Device, TextureFormat, TextureView};

use ab_glyph::FontArc;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Layout, Section, Text};

pub struct FpsText {
    staging_belt: StagingBelt,
    brush: GlyphBrush<()>,
}

impl FpsText {
    pub fn new(device: &Device, render_format: TextureFormat) -> Self {
        let staging_belt = StagingBelt::new(1024);
        let font =
            FontArc::try_from_slice(include_bytes!("../fonts/LiberationMono-Bold.ttf")).unwrap();

        let brush = GlyphBrushBuilder::using_font(font).build(device, render_format);

        Self {
            staging_belt,
            brush,
        }
    }

    pub fn draw(
        &mut self,
        fps: f32,
        device: &Device,
        encoder: &mut CommandEncoder,
        target: &TextureView,
    ) {
        let text = format!("FPS: {:.1}", fps);
        let text_render = Text::new(&text)
            .with_color([0.0, 1.0, 1.0, 1.0])
            .with_scale(16.0);

        let layout = Layout::default();
        let section = Section::default().add_text(text_render).with_layout(layout);

        self.brush.queue(section);

        self.brush
            .draw_queued(device, &mut self.staging_belt, encoder, target, 400, 200)
            .unwrap();
    }

    pub fn submit(&mut self) {
        self.staging_belt.finish();
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
    }
}
