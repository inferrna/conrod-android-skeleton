extern crate android_ndk;
#[macro_use] extern crate conrod_core;
#[macro_use] extern crate conrod_winit;
#[macro_use] extern crate conrod_glium;
extern crate image;
extern crate rand;
extern crate ndk;
extern crate rusttype;
extern crate glium;
extern crate winit;


mod app;
mod assets;
#[macro_use] mod macros;

use glium::Surface;
use glium::glutin::window::WindowBuilder;
use android_ndk::android_app::AndroidApp;
use android_ndk::native_activity::NativeActivity;
use conrod_winit::WinitWindow;
use conrod_glium::{TextureDimensions, Display};
use glium::glutin::event::Event;
use glium::backend::glutin::glutin::window::Window;
use std::thread;
use std::time::Duration;
use glium::backend::glutin::glutin::GlProfile;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
pub fn main() {
    eprintln!("Call to main(..)");
    let native_activity = ndk_glue::native_activity();
    eprintln!("got native_activity");
    /*let vm_ptr = native_activity.vm();
    eprintln!("got vm_ptr");
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.expect("Can't get VM from activity");
    eprintln!("got vm");
    let env = vm.attach_current_thread().expect("Can't attach VM to current thread");;
    eprintln!("got env");*/

    //let android_app = unsafe { AndroidApp::from_ptr(ndk_glue:: get_android_app()) };
    //let native_activity: NativeActivity = android_app.activity();
    let builder = glium::glutin::window::WindowBuilder::new().with_title("Hello conrod");
    eprintln!("got builder");
    let context = glium::glutin::ContextBuilder::new()
        .with_srgb(false)
        .with_pixel_format(24u8, 0u8)
        .with_depth_buffer(16)
        .with_gl_debug_flag(true)
        .with_stencil_buffer(0)
        .with_gl_profile(GlProfile::Core)
        .with_gl(glium::glutin::GlRequest::Specific(glium::glutin::Api::OpenGlEs, (2, 0)))
        .with_hardware_acceleration(Some(false))
        .with_vsync(false);
    eprintln!("got context");
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    eprintln!("got events_loop");
    //let mut events_loop = winit::EventsLoop::new();
    thread::sleep(Duration::from_millis(2500));
    assert!(ndk_glue::native_window().is_some());

    /*let window_context = context.build_windowed(builder, &events_loop).unwrap();
    eprintln!("got window_context 1");

    let window_context = unsafe { window_context.make_current().unwrap() };
    eprintln!("got window_context 2");

    let window = window_context.window();*/

    let display = glium::Display::new(builder, context, &events_loop).unwrap();
    //let display = Window::new(&events_loop).unwrap();
    eprintln!("got display");

    let (w, h) = display.get_framebuffer_dimensions();
    //let (w, h) = window.inner_size().into();
    eprintln!("got (w, h)");
    let mut ui = conrod_core::UiBuilder::new([w as f64, h as f64]).theme(app::theme()).build();
    eprintln!("got ui");
    ui.fonts.insert(assets::load_font(native_activity, "LiberationSans-Regular.ttf"));

    let mut image_map: conrod_core::image::Map<glium::texture::Texture2d> = conrod_core::image::Map::new();
    let image_rgba = assets::load_image(native_activity,"rust.png").unwrap().to_rgba8();
    let dims = image_rgba.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image_rgba.into_raw(), dims);
    let texture = glium::texture::Texture2d::new(&display, raw_image).unwrap();
    let rust_logo = image_map.insert(texture);

    let mut demo_app = app::DemoApp::new(rust_logo);
    let ids = app::Ids::new(ui.widget_id_generator());

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (render_tx, render_rx) = std::sync::mpsc::channel();
    let events_loop_proxy = events_loop.create_proxy();


    std::thread::spawn(move || {
        eprintln!("Spawn events handling thread");
        let mut needs_update = true;
        let mut first = true;
        loop {
            if !first {
                eprintln!("Enter !first at events handling thread");
                let mut events = Vec::new();
                while let Ok(event) = event_rx.try_recv() {
                    events.push(event);
                }

                if events.is_empty() || !needs_update {
                    match event_rx.recv() {
                        Ok(event) => events.push(event),
                        Err(_) => break
                    }
                }

                needs_update = false;

                for event in events {
                    ui.handle_event(event);
                    needs_update = true;
                }
            }
            else {
                first = false;
            }

            eprintln!("Drawing the GUI.");
            app::gui(&mut ui.set_widgets(), &ids, &mut demo_app);

            if let Some(primitives) = ui.draw_if_changed() {
                if render_tx.send(primitives.owned()).is_err() {
                    break;
                }
            }
        }
    });

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();
    let mut last_update = std::time::Instant::now();
    let mut closed = false;
    let mut first = true;
    while !closed {
        eprintln!("Enter main loop");
        let primitives: Option<conrod_core::render::OwnedPrimitives>;

        if !first {
            eprintln!("Enter !first at main loop");
            // Don't loop more rapidly than 60Hz.
            let sixteen_ms = std::time::Duration::from_millis(16);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }

            events_loop.run( move |event, _, _| { //: Event<'a, ()>
                //let evt = event.clone();//.into();

                let in_evt = convert_event!(&event, &display);
                if let Some(event) = in_evt {
                    event_tx.send(event).unwrap();
                }

                let evt_res = match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        glium::glutin::event::WindowEvent::CloseRequested|glium::glutin::event::WindowEvent::Destroyed => {
                            eprintln!("Got WindowEvent CloseRequested");
                            closed = true;
                            glium::glutin::event_loop::ControlFlow::Exit
                        },
                        glium::glutin::event::WindowEvent::Resized(..) => {
                            eprintln!("Got WindowEvent Resized");
                            if let Some(primitives) = render_rx.iter().next() {
                                draw(&display, &mut renderer, &image_map, &primitives);
                            }
                            glium::glutin::event_loop::ControlFlow::Poll
                        },
                        e => {
                            eprintln!("Got WindowEvent '{:?}' - unhandled", e);
                            glium::glutin::event_loop::ControlFlow::Poll
                        },
                    },
                    glium::glutin::event::Event::Resumed => {
                        eprintln!("Got Event Resumed");
                        glium::glutin::event_loop::ControlFlow::Exit
                    },
                    e => {
                        eprintln!("Got Event '{:?}' - unhandled", e);
                        glium::glutin::event_loop::ControlFlow::Poll
                    },
                };
                //evt_res
            });

            primitives = render_rx.try_iter().last();
        }
        else {
            first = false;
            primitives = render_rx.recv().ok();
        }

        if let Some(primitives) = primitives {
            eprintln!("Rendering.");
            draw(&display, &mut renderer, &image_map, &primitives);
        }

        last_update = std::time::Instant::now();
    }
}

fn draw(display: &glium::Display,
        renderer: &mut conrod_glium::Renderer,
        image_map: &conrod_core::image::Map<glium::Texture2d>,
        primitives: &conrod_core::render::OwnedPrimitives) {
    eprintln!("Call to draw(..)");
    renderer.fill(display, primitives.walk(), &image_map);
    let mut target = display.draw();
    target.clear_color(1.0, 1.0, 1.0, 1.0);
    renderer.draw(display, &mut target, &image_map).unwrap();
    target.finish().unwrap();
}
