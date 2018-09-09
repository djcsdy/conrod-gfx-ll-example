#[cfg(all(windows, not(feature = "vulkan")))]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(any(all(not(windows), not(target_os = "macos")), all(windows, feature = "vulkan")))]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal;
extern crate winit;

fn main() {
    println!("Hello, world!");
}
