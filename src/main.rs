#[cfg(all(windows, not(feature = "vulkan")))]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(any(all(not(windows), not(target_os = "macos")), all(windows, feature = "vulkan")))]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal;
extern crate winit;

fn main() {
    let mut events_loop = winit::EventsLoop::new();

    let window = winit::WindowBuilder::new()
        .with_dimensions(winit::dpi::LogicalSize::from_physical(
            winit::dpi::PhysicalSize {
                width: 600.0,
                height: 420.0,
            },
            1.0,
        ))
        .with_title("Conrod gfx-ll example")
        .build(&events_loop)
        .unwrap();

    let instance = gfx_backend::Instance::create("conrod gfx-ll example", 0);

    let mut surface = instance.create_surface(&window);
}
