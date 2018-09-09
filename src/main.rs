#[cfg(all(windows, not(feature = "vulkan")))]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(any(all(not(windows), not(target_os = "macos")), all(windows, feature = "vulkan")))]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal;
extern crate winit;

use gfx_hal::device::Device;
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::capability::Graphics;
use gfx_hal::queue::capability::Transfer;
use gfx_hal::queue::family::QueueFamily;
use gfx_hal::queue::QueueType;
use gfx_hal::Gpu;
use gfx_hal::Instance;
use gfx_hal::PhysicalDevice;
use gfx_hal::Surface;

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

    let mut adapter = {
        let mut adapters = instance.enumerate_adapters();

        adapters.sort_by_key(|adapter| match adapter.info.device_type {
            gfx_hal::adapter::DeviceType::IntegratedGpu => 0,
            gfx_hal::adapter::DeviceType::DiscreteGpu => 1,
            gfx_hal::adapter::DeviceType::VirtualGpu => 2,
            gfx_hal::adapter::DeviceType::Cpu => 3,
            gfx_hal::adapter::DeviceType::Other => 4,
        });

        adapters.remove(0)
    };

    let presentation_queue_family = adapter
        .queue_families
        .iter()
        .find(|family| surface.supports_queue_family(family))
        .unwrap();

    let transfer_queue_family = adapter
        .queue_families
        .iter()
        .find(|family| family.queue_type() == QueueType::Transfer)
        .unwrap_or(presentation_queue_family);

    let Gpu { device, mut queues } = adapter
        .physical_device
        .open(&[
            (&transfer_queue_family, &[1.0]),
            (&presentation_queue_family, &[1.0]),
        ])
        .unwrap();

    let transfer_queue_group = queues.take::<Transfer>(transfer_queue_family.id()).unwrap();

    let presentation_queue_group = queues
        .take::<Graphics>(presentation_queue_family.id())
        .unwrap();

    let mut transfer_command_pool =
        device.create_command_pool_typed(&transfer_queue_group, CommandPoolCreateFlags::empty(), 1);

    let mut presentation_command_pool = device.create_command_pool_typed(
        &presentation_queue_group,
        CommandPoolCreateFlags::empty(),
        1,
    );
}
