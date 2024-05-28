use std::{ffi::CString, mem::size_of, num::NonZeroU32};

use glium::{
    glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextApi, ContextAttributesBuilder, NotCurrentContext, NotCurrentGlContext},
        display::{GetGlDisplay, GlDisplay},
        surface::{SurfaceAttributesBuilder, WindowSurface},
    },
    Surface,
};
use imgui_winit_support::{
    winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    },
    WinitPlatform,
};
use implot::{
    sys::{ImPlot_BeginPlot, ImPlot_EndPlot, ImPlot_PlotLine_FloatPtrInt},
    ImVec2,
};
use raw_window_handle::HasRawWindowHandle;

fn create_window<T: Into<String>>(
    title: T,
) -> (
    EventLoop<()>,
    imgui_winit_support::winit::window::Window,
    glium::Display<WindowSurface>,
) {
    let event_loop = EventLoop::new().unwrap();

    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(800, 600));

    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .unwrap();
    let window = window.unwrap();

    let mut context: Option<NotCurrentContext> = None;
    for api in [ContextApi::OpenGl(None), ContextApi::Gles(None)].iter() {
        if context.is_some() {
            break;
        }

        let context_attribs = ContextAttributesBuilder::new()
            .with_context_api(*api)
            .with_profile(glium::glutin::context::GlProfile::Core)
            .build(Some(window.raw_window_handle()));

        context = unsafe { cfg.display().create_context(&cfg, &context_attribs).ok() };
    }
    let context = context.unwrap();

    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(800).unwrap(),
        NonZeroU32::new(600).unwrap(),
    );
    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &surface_attribs)
            .expect("Failed to create OpenGL surface")
    };

    let context = context
        .make_current(&surface)
        .expect("Failed to make OpenGL context current");

    let display = glium::Display::from_context_surface(context, surface)
        .expect("Failed to create glium Display");

    (event_loop, window, display)
}

fn imgui_init(
    window: &imgui_winit_support::winit::window::Window,
) -> (WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    let dpi_mode = imgui_winit_support::HiDpiMode::Default;

    winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    (winit_platform, imgui_context)
}

fn main() {
    let (event_loop, window, display) = create_window("Hello ImPlot!");
    let (mut winit_platform, mut imgui_context) = imgui_init(&window);

    let _plot = implot::Context::create();

    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display).unwrap();

    event_loop
        .run(move |event, window_target| match event {
            Event::NewEvents(_) => {}
            Event::AboutToWait => {
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), &window)
                    .expect("Failed to prepare frame");
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Create frame
                let ui = imgui_context.frame();

                ui.window("Test Window").build(|| {
                    let title_id = CString::new("A plot").unwrap();
                    let size = ImVec2 { x: 0.0, y: 0.0 };
                    // let flags = ImPlotFlags__ImPlotFlags_None as i32;
                    let flags = 0;
                    let plot_id = CString::new("My plot").unwrap();
                    unsafe {
                        let should_render = ImPlot_BeginPlot(title_id.as_ptr(), size, flags);
                        if should_render {
                            ImPlot_PlotLine_FloatPtrInt(
                                plot_id.as_ptr(),
                                [0.0, 1.0, 2.0, 4.0].as_ptr(),
                                4,
                                1.0,
                                0.0,
                                0,
                                0,
                                size_of::<f32>() as i32,
                            );
                            ImPlot_EndPlot();
                        }
                    }
                });

                // Setup drawing
                let mut target = display.draw();

                // Clear screen
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);

                // Render
                winit_platform.prepare_render(ui, &window);
                let draw_data = imgui_context.render();
                renderer.render(&mut target, draw_data).unwrap();
                target.finish().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window_target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    display.resize((new_size.width, new_size.height));
                }
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
            event => winit_platform.handle_event(imgui_context.io_mut(), &window, &event),
        })
        .unwrap();
}
