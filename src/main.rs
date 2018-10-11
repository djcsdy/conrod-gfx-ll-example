#[macro_use]
extern crate conrod;
#[cfg(all(windows, not(feature = "vulkan")))]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(target_os = "macos")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(any(all(not(windows), not(target_os = "macos")), all(windows, feature = "vulkan")))]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal;
#[macro_use]
extern crate glsl_to_spirv_macros;
#[macro_use]
extern crate glsl_to_spirv_macros_impl;
extern crate rand;
extern crate rusttype;
extern crate winit;

mod gui;
mod renderer;
mod theme;

use gfx_hal::device::Device;
use gfx_hal::format::ChannelType;
use gfx_hal::format::Format;
use gfx_hal::image::{Layout, Usage};
use gfx_hal::pass::{
    Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDependency, SubpassDesc,
};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::queue::capability::Graphics;
use gfx_hal::queue::family::QueueFamily;
use gfx_hal::window::AcquireError;
use gfx_hal::window::Extent2D;
use gfx_hal::window::FrameSync;
use gfx_hal::window::PresentMode;
use gfx_hal::window::Swapchain;
use gfx_hal::window::SwapchainConfig;
use gfx_hal::Backend;
use gfx_hal::Gpu;
use gfx_hal::Instance;
use gfx_hal::PhysicalDevice;
use gfx_hal::Surface;
use rusttype::FontCollection;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

const WIDTH: i32 = 600;
const HEIGHT: i32 = 420;

fn main() {
    let (events_sender, events_receiver) = channel();

    let (window_thread, (window, instance, mut surface)) = {
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
                .map(Arc::new)
                .unwrap();

            let surface = instance.create_surface(&window);

            window_instance_surface_sender
                .send((Arc::downgrade(&window), instance, surface))
                .unwrap();

            events_loop.run_forever(|event| match event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => winit::ControlFlow::Break,
                event => {
                    if let Some(conrod_event) =
                        conrod::backend::winit::convert_event(event, &*window)
                    {
                        events_sender.send(conrod_event).unwrap();
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

    let (surface_capabilities, surface_formats, mut present_modes) =
        surface.compatibility(&adapter.physical_device);

    let surface_format = surface_formats
        .map(|formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        })
        .unwrap_or(Format::Rgb8Srgb);

    present_modes.sort_by_key(|mode| match mode {
        PresentMode::Mailbox => 0,
        PresentMode::Relaxed => 1,
        PresentMode::Fifo => 2,
        PresentMode::Immediate => 3,
    });

    let present_mode = present_modes[0];

    let graphics_queue_family = adapter
        .queue_families
        .iter()
        .find(|family| surface.supports_queue_family(family))
        .unwrap();

    let Gpu { device, mut queues } = adapter
        .physical_device
        .open(&[(&graphics_queue_family, &[1.0])])
        .unwrap();

    let mut swapchain = build_swapchain::<gfx_backend::Backend>(
        &*window.upgrade().unwrap(),
        &adapter.physical_device,
        &device,
        &mut surface,
        surface_format,
        present_mode,
    );

    let mut render_pass = build_render_pass::<gfx_backend::Backend>(&device, surface_format);

    let mut graphics_queue_group = queues.take::<Graphics>(graphics_queue_family.id()).unwrap();

    let mut graphics_command_pool =
        device.create_command_pool_typed(&graphics_queue_group, CommandPoolCreateFlags::empty(), 1);

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

    let frame_semaphore = device.create_semaphore();

    'main: loop {
        gui::render(&mut ui.set_widgets(), &ids, &mut state);

        if let Some(primitives) = ui.draw_if_changed() {
            if match swapchain.acquire_image(4000, FrameSync::Semaphore(&frame_semaphore)) {
                Ok(swapchain_image_index) => graphics_queue_group.queues[0].present(
                    vec![(&swapchain, swapchain_image_index)],
                    vec![&frame_semaphore],
                ),
                Err(AcquireError::NotReady) => Ok(()),
                Err(_) => Err(()),
            }.is_err()
            {
                device.destroy_swapchain(swapchain);

                swapchain = build_swapchain::<gfx_backend::Backend>(
                    &*window.upgrade().unwrap(),
                    &adapter.physical_device,
                    &device,
                    &mut surface,
                    surface_format,
                    present_mode,
                );

                ui.needs_redraw();
            };
        }

        let mut event_option = match events_receiver.recv() {
            Result::Ok(event) => Some(event),
            Result::Err(_) => break 'main,
        };

        while let Some(event) = event_option {
            ui.handle_event(event);
            event_option = events_receiver.try_recv().ok();
        }
    }

    device.destroy_semaphore(frame_semaphore);

    device.destroy_render_pass(render_pass);

    device.destroy_swapchain(swapchain);

    window_thread.join().unwrap();
}

fn build_render_pass<B: Backend>(
    device: &<B as gfx_hal::Backend>::Device,
    surface_format: Format,
) -> <B as gfx_hal::Backend>::RenderPass {
    device.create_render_pass(
        vec![Attachment {
            format: Some(surface_format),
            samples: 1,
            ops: AttachmentOps {
                load: AttachmentLoadOp::Clear,
                store: AttachmentStoreOp::Store,
            },
            stencil_ops: AttachmentOps {
                load: AttachmentLoadOp::DontCare,
                store: AttachmentStoreOp::DontCare,
            },
            layouts: Layout::Undefined..Layout::General,
        }],
        vec![SubpassDesc {
            colors: &[(0, Layout::General)],
            depth_stencil: None,
            inputs: &[],
            resolves: &[],
            preserves: &[],
        }],
        vec![] as Vec<SubpassDependency>,
    )
}

fn build_swapchain<B: Backend>(
    window: &winit::Window,
    physical_device: &<B as gfx_hal::Backend>::PhysicalDevice,
    device: &<B as gfx_hal::Backend>::Device,
    surface: &mut <B as gfx_hal::Backend>::Surface,
    surface_format: Format,
    present_mode: PresentMode,
) -> <B as gfx_hal::Backend>::Swapchain {
    let (capabilities, _, _) = surface.compatibility(physical_device);

    let extent = match capabilities.current_extent {
        Some(extent) => extent,
        None => {
            let window_size = window
                .get_inner_size()
                .unwrap()
                .to_physical(window.get_hidpi_factor());
            let mut extent = Extent2D {
                width: window_size.width as _,
                height: window_size.height as _,
            };
            extent.width = extent
                .width
                .max(capabilities.extents.start.width)
                .min(capabilities.extents.end.width);
            extent.height = extent
                .height
                .max(capabilities.extents.start.height)
                .min(capabilities.extents.end.height);
            extent
        }
    };

    let image_count = 2.max(capabilities.image_count.start)
        .min(capabilities.image_count.end);

    let config = SwapchainConfig::new(extent.width, extent.height, surface_format, image_count)
        .with_image_usage(Usage::TRANSFER_DST)
        .with_mode(present_mode);

    device.create_swapchain(surface, config, None).0
}
