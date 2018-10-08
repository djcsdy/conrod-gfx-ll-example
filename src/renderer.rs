use gfx_hal::command::OneShot;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::capability::Graphics;
use gfx_hal::Backend;

pub fn render<B: Backend>(
    graphics_command_pool: &mut CommandPool<B, Graphics>,
) {
    let mut graphics_command_buffer =
        graphics_command_pool.acquire_command_buffer::<OneShot>(false);
}
