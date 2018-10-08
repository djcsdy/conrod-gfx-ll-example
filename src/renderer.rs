use conrod::render::Primitive;
use conrod::render::PrimitiveKind;
use gfx_hal::command::CommandBuffer;
use gfx_hal::command::OneShot;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::capability::Graphics;
use gfx_hal::Backend;

pub fn render<B: Backend>(graphics_command_pool: &mut CommandPool<B, Graphics>) {
    let mut graphics_command_buffer =
        graphics_command_pool.acquire_command_buffer::<OneShot>(false);
}

pub fn render_primitive<B: Backend>(
    command_buffer: &mut CommandBuffer<B, Graphics>,
    primitive: &Primitive,
) {
    match &primitive.kind {
        PrimitiveKind::Rectangle { color } => (),
        PrimitiveKind::TrianglesSingleColor { triangles, color } => (),
        PrimitiveKind::TrianglesMultiColor { triangles } => (),
        PrimitiveKind::Image {
            image_id,
            color,
            source_rect,
        } => (),
        PrimitiveKind::Text {
            color,
            text,
            font_id,
        } => (),
        PrimitiveKind::Other(_) => (),
    }
}
