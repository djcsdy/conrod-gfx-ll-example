#[macro_use]
extern crate conrod;
#[cfg(all(windows, not(feature = "vulkan")))]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(any(all(not(windows), not(target_os = "macos")), all(windows, feature = "vulkan")))]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal;
extern crate rusttype;
extern crate winit;

mod gui;
mod renderer;
mod theme;

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
use rusttype::FontCollection;
use std::sync::mpsc::channel;
use std::thread;

const WIDTH: i32 = 600;
const HEIGHT: i32 = 420;

fn main() {
    let (events_sender, events_receiver) = channel();

    let (window_thread, (instance, mut surface)) = {
        let instance = gfx_backend::Instance::create("conrod gfx-ll example", 0);
        let (window_instance_surface_sender, window_instance_surface_receiver) = channel();

        let window_thread = thread::spawn(move || {
            let mut events_loop = winit::EventsLoop::new();

            let window = winit::WindowBuilder::new()
                .with_dimensions(winit::dpi::LogicalSize::from_physical(
                    winit::dpi::PhysicalSize {
                        width: WIDTH as f64,
                        height: HEIGHT as f64,
                    },
                    1.0,
                ))
                .with_title("Conrod gfx-ll example")
                .build(&events_loop)
                .unwrap();

            let surface = instance.create_surface(&window);

            window_instance_surface_sender
                .send((instance, surface))
                .unwrap();

            events_loop.run_forever(|event| match event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => winit::ControlFlow::Break,
                event => {
                    if let Some(conrod_event) =
                        conrod::backend::winit::convert_event(event, &window)
                    {
                        events_sender.send(conrod_event);
                    }
                    winit::ControlFlow::Continue
                }
            });
        });

        (
            window_thread,
            window_instance_surface_receiver.recv().unwrap(),
        )
    };

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

    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
        .theme(theme::theme())
        .build();

    FontCollection::from_bytes(include_bytes!("NotoSans-Regular.ttf") as &[u8])
        .unwrap()
        .into_fonts()
        .for_each(|font| {
            ui.fonts.insert(font.unwrap());
        });

    // TODO: Load a real image instead of an empty placeholder value.
    let mut image_map = conrod::image::Map::<()>::new();
    let rust_logo = image_map.insert(());

    let mut state = gui::State::new(rust_logo);

    let ids = gui::Ids::new(ui.widget_id_generator());

    window_thread.join().unwrap();
}
