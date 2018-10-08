use gfx_hal::pool::CommandPool;
use gfx_hal::queue::capability::Graphics;
use gfx_hal::Backend;

pub fn render<B: Backend>(
    graphics_command_pool: CommandPool<B, Graphics>,
) {
}
