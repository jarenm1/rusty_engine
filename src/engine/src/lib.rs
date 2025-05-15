pub mod prelude;

use std::{ptr::with_exposed_provenance, sync::Arc};
use winit::{application::ApplicationHandler, event::ElementState, event_loop::EventLoop, keyboard::Key, window::{Window, WindowAttributes}};
use rendering::renderer::Renderer;
use jaren_ecs::system::System;
use web_sys::console::log_1;

#[derive(Default)]
pub struct GameConfig {
    pub title: String,
}

pub enum FunctionMode {
    Startup,
    Update,
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    config: GameConfig,
    ecs: System,
    startup_systems: Vec<Box<dyn FnMut(&mut System)>>,
    update_systems: Vec<Box<dyn FnMut(&mut System)>>,
}

impl App {
    pub fn new(config: GameConfig) -> Self {
        Self {
            window: None,
            renderer: None,
            config,
            ecs: System::new(),
            startup_systems: Vec::new(),
            update_systems: Vec::new(),
        }
    }
    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();

        #[cfg(target_arch = "wasm32")]
        use winit::platform::web::EventLoopExtWebSys;
        #[cfg(target_arch = "wasm32")]
        event_loop.spawn_app(self);

        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(&mut self).expect("Failed to run event loop");
    }
    pub fn add_system<F>(mut self, mode: FunctionMode, func: F) -> Self
    where
        F: FnMut(&mut System) + 'static,
    {
        match mode {
            FunctionMode::Startup => self.startup_systems.push(Box::new(func)),
            FunctionMode::Update => self.update_systems.push(Box::new(func)),
        }
        self
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = {
                #[cfg(target_arch = "wasm32")]
                {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    use web_sys::{HtmlCanvasElement, window};
                    use wasm_bindgen::JsCast;
                    
                    let canvas = {
                        log_1(&"resumed: Getting window and document".into());
                        let window = window().expect("Failed to get window");
                        let document = window.document().expect("Failed to get document");
                        let body = document.body().expect("Document should have a body");
                        log_1(&"resumed: Got window, document, body".into());

                        // Create canvas element
                        log_1(&"resumed: Creating canvas element".into());
                        let canvas_element = document
                            .create_element("canvas")
                            .expect("Failed to create canvas element")
                            .dyn_into::<HtmlCanvasElement>()
                            .map_err(|_| ())
                            .expect("Failed to cast element to HtmlCanvasElement");
                        log_1(&"resumed: Canvas element created".into());
                        
                        canvas_element.set_id("game-canvas-created-by-rust");

                        // Append canvas to the document body
                        log_1(&"resumed: Appending canvas to body".into());
                        body.append_child(&canvas_element)
                            .expect("Failed to append canvas to body");
                        log_1(&"resumed: Canvas appended to body".into());

                        canvas_element 
                    };
                    
                    log_1(&"resumed: Creating winit window with canvas".into());
                    let mut window_attrs = WindowAttributes::default();
                    window_attrs = window_attrs
                        .with_title(&self.config.title)
                        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                        .with_canvas(Some(canvas)); // Use the dynamically created canvas
                    log_1(&"resumed: Winit window created".into()); // Log before storing
                    event_loop.create_window(window_attrs).unwrap()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let window_attributes = WindowAttributes::default()
                        .with_title(&self.config.title);
                    event_loop
                        .create_window(window_attributes)
                        .expect("failed to create window")
                }
            };

            let window_arc = Arc::new(window); // Create Arc<Window>
            self.window = Some(window_arc.clone()); // Store the Arc

            self.renderer = Some(pollster::block_on(Renderer::new(window_arc.clone(), "25010123242900.jpeg")));

            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize(renderer.size()); // Call resize with initial size
            }

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console::log_1;
                log_1(&"resumed: Renderer initialized and configured".into()); // Update log
            }
        }
        // Run all startup systems after ECS and renderer are ready
        for system in &mut self.startup_systems {
            (system)(&mut self.ecs);
        }
    }
    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
    ) {
        if self.window.as_ref().map_or(false, |w| w.id() == window_id) {

            let renderer = match self.renderer.as_mut() {
                Some(r) => r,
                None => return, 
            };

            match event {
                winit::event::WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                winit::event::WindowEvent::Resized(physical_size) => {
                     renderer.resize(physical_size);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    // Run all update systems per frame
                    for system in &mut self.update_systems {
                        (system)(&mut self.ecs);
                    }
                    renderer.update();
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size()), // Use getter method
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Error rendering frame: {:?}", e),
                    }
                     if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                     }
                }
                winit::event::WindowEvent::KeyboardInput { event, .. } => { 
                    if let Key::Named(named_key) = event.logical_key {
                        match event.state {
                            ElementState::Pressed => {
                            }
                            ElementState::Released => {
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw(); // Request redraw on the Arc
        }
    }
}
