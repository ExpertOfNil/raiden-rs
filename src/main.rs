use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use raiden_rs::{
    camera::PanOrbitCamera,
    commands::{DrawCommand, DrawCommandBuilder},
    mesh::MeshType,
};
use std::sync::Arc;
use std::{any::Any, collections::BTreeMap};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
pub struct MouseState {
    pub button_left: bool,
    pub button_right: bool,
    pub button_middle: bool,
    pub position: glam::Vec2,
    pub position_needs_update: bool,
    pub touches: BTreeMap<u64, PhysicalPosition<f64>>,
}

pub struct State {
    is_surface_configured: bool,
    is_scene_initialized: bool,
    window: Arc<Window>,
    pub renderer: raiden_rs::renderer::Renderer,
    pub mouse_state: MouseState,
    pub camera: PanOrbitCamera,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let mut renderer = raiden_rs::renderer::Renderer::from_winit(window.clone()).await?;
        let camera = PanOrbitCamera::default();
        renderer.update_uniforms(&camera);

        Ok(Self {
            is_surface_configured: false,
            is_scene_initialized: false,
            window,
            renderer,
            mouse_state: MouseState::default(),
            camera,
        })
    }

    pub fn ensure_scene_initialized(&mut self) {
        if self.is_scene_initialized || !self.is_surface_configured {
            return;
        }
        log::debug!("Initializing Scene");
        self.renderer.commands.push(
            DrawCommandBuilder::new(MeshType::Sphere)
                .with_position([0.0, 0.0, 0.0].into())
                .with_scale(0.5)
                .with_color_u8(255, 255, 255, 255)
                .build(),
        );
        self.renderer.commands.push(
            DrawCommandBuilder::new(MeshType::Cube)
                .with_position([4.0, 0.0, 0.0].into())
                .with_scale(0.1)
                .with_color_u8(255, 0, 0, 255)
                .build(),
        );
        self.renderer.commands.push(
            DrawCommandBuilder::new(MeshType::Cube)
                .with_position([0.0, 4.0, 0.0].into())
                .with_scale(0.1)
                .with_color_u8(0, 255, 0, 255)
                .build(),
        );
        self.renderer.commands.push(
            DrawCommandBuilder::new(MeshType::Cube)
                .with_position([0.0, 0.0, 4.0].into())
                .with_scale(0.1)
                .with_color_u8(0, 0, 255, 255)
                .build(),
        );
        self.is_scene_initialized = true;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // Needed for firefox
            const MAX_DIM: u32 = 2048;
            let scale = if width > MAX_DIM || height > MAX_DIM {
                MAX_DIM as f32 / width.max(height) as f32
            } else {
                1.0
            };
            self.renderer.surface_config.width = (width as f32 * scale) as u32;
            self.renderer.surface_config.height = (height as f32 * scale) as u32;
            self.renderer
                .surface
                .configure(&self.renderer.device, &self.renderer.surface_config);
            self.is_surface_configured = true;

            let window_size = glam::UVec2::new(
                self.renderer.surface_config.width,
                self.renderer.surface_config.height,
            );
            self.renderer.update_depth_texture(window_size);
            log::debug!(
                "Window Size: {}x{}",
                self.renderer.surface_config.width,
                self.renderer.surface_config.height
            );
            self.ensure_scene_initialized();
            self.camera.update_aspect(window_size);
            self.renderer.update_uniforms(&self.camera);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let window_size = glam::UVec2::new(
            self.renderer.surface_config.width,
            self.renderer.surface_config.height,
        );
        self.ensure_scene_initialized();
        self.window.request_redraw();
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.renderer.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // TODO (mmckenna) : move to renderer
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.renderer.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.renderer.solid_pipeline);
            render_pass.set_bind_group(0, &self.renderer.uniform_bind_group, &[]);

            // Draw meshes
            let mesh_types: Vec<MeshType> = self.renderer.meshes.keys().cloned().collect();
            for mesh_type in mesh_types {
                match mesh_type {
                    MeshType::Cube => self.renderer.render_mesh(&mesh_type, &mut render_pass),
                    MeshType::Tetrahedron => {
                        self.renderer.render_mesh(&mesh_type, &mut render_pass)
                    }
                    MeshType::Sphere => {
                        self.renderer.render_mesh(&mesh_type, &mut render_pass)
                    }
                    _ => log::warn!(
                        "{:?} mesh rendering has not been implemented yet",
                        mesh_type
                    ),
                }
            }
        }

        self.renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<State>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "the_canvas_id";
            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));

            if let Some(loading_text) = document.get_element_by_id("loading_text") {
                loading_text.remove();
            }
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = Some(pollster::block_on(State::new(window)).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(State::new(window).await.expect("Unable to create canvas"))
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, mut event: State) {
        #[cfg(target_arch = "wasm32")]
        {
            let window_size = event.window.inner_size();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
            event.window.request_redraw();
        }
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let app_state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => app_state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match app_state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    let size = app_state.window.inner_size();
                    app_state.resize(size.width, size.height);
                }
                Err(e) => {
                    log::error!("Unable to render {}", e);
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => app_state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => app_state.mouse_state.button_left = state.is_pressed(),
                    MouseButton::Middle => app_state.mouse_state.button_middle = state.is_pressed(),
                    MouseButton::Right => app_state.mouse_state.button_right = state.is_pressed(),
                    _ => {}
                };
                app_state.mouse_state.position_needs_update = true;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, vert) => app_state.camera.zoom(vert),
                    MouseScrollDelta::PixelDelta(delta) => app_state.camera.zoom(delta.y as f32),
                }
                app_state.renderer.update_uniforms(&app_state.camera);
            }
            WindowEvent::Touch(Touch {
                id,
                location,
                phase,
                ..
            }) => match phase {
                TouchPhase::Started => {
                    if app_state.mouse_state.touches.insert(id, location).is_none()
                        && app_state.mouse_state.touches.len() == 1
                    {
                        app_state.mouse_state.position.x = location.x as f32;
                        app_state.mouse_state.position.y = location.y as f32;
                    }
                }
                TouchPhase::Moved => {
                    if let Some(prev_pos) = app_state.mouse_state.touches.insert(id, location) {
                        if app_state.mouse_state.touches.len() == 2 {
                            // 2-finger pinch zoom
                            let other_id: u64 = app_state
                                .mouse_state
                                .touches
                                .keys()
                                .cloned()
                                .find(|oid| oid != &id)
                                .unwrap();
                            let other_pos = app_state.mouse_state.touches.get(&other_id).unwrap();
                            let other_loc = glam::vec2(other_pos.x as f32, other_pos.y as f32);
                            let prev_loc = glam::vec2(prev_pos.x as f32, prev_pos.y as f32);
                            let prev_spc = other_loc.distance(prev_loc);
                            let curr_loc = glam::vec2(location.x as f32, location.y as f32);
                            let curr_spc = other_loc.distance(curr_loc);
                            app_state.camera.zoom((curr_spc - prev_spc) * 0.2);
                            app_state.renderer.update_uniforms(&app_state.camera);
                            let primary_touch_key =
                                *app_state.mouse_state.touches.first_entry().unwrap().key();
                            if id == primary_touch_key {
                                let curr_pos = glam::vec2(location.x as f32, location.y as f32);
                                app_state.mouse_state.position.x = curr_pos.x;
                                app_state.mouse_state.position.y = curr_pos.y;
                            }
                        } else if app_state.mouse_state.touches.len() == 3 {
                            // 3-finger touch panning
                            let primary_touch_key =
                                *app_state.mouse_state.touches.first_entry().unwrap().key();
                            if id == primary_touch_key {
                                let curr_pos = glam::vec2(location.x as f32, location.y as f32);
                                let prev_pos = glam::vec2(prev_pos.x as f32, prev_pos.y as f32);
                                app_state.camera.pan(curr_pos - prev_pos);
                                app_state.renderer.update_uniforms(&app_state.camera);
                                app_state.mouse_state.position.x = curr_pos.x;
                                app_state.mouse_state.position.y = curr_pos.y;
                            }
                        } else {
                            // 1-finger orbit
                            let delta = glam::vec2(
                                (location.x - prev_pos.x) as f32,
                                (location.y - prev_pos.y) as f32,
                            );
                            app_state.camera.orbit(delta);
                            app_state.renderer.update_uniforms(&app_state.camera);
                            app_state.mouse_state.position.x = location.x as f32;
                            app_state.mouse_state.position.y = location.y as f32;
                        }
                    }
                }
                TouchPhase::Ended => {
                    app_state.mouse_state.touches.remove(&id);
                }
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                if app_state.mouse_state.position_needs_update {
                    app_state.mouse_state.position.x = position.x as f32;
                    app_state.mouse_state.position.y = position.y as f32;
                    app_state.mouse_state.position_needs_update = false;
                } else {
                    if app_state.mouse_state.button_left {
                        let mouse_delta = [
                            position.x as f32 - app_state.mouse_state.position.x,
                            position.y as f32 - app_state.mouse_state.position.y,
                        ]
                        .into();
                        app_state.camera.orbit(mouse_delta);
                        app_state.renderer.update_uniforms(&app_state.camera);
                        app_state.mouse_state.position.x = position.x as f32;
                        app_state.mouse_state.position.y = position.y as f32;
                    }
                    if app_state.mouse_state.button_right {
                        let mouse_delta = [
                            position.x as f32 - app_state.mouse_state.position.x,
                            position.y as f32 - app_state.mouse_state.position.y,
                        ]
                        .into();
                        app_state.camera.pan(mouse_delta);
                        app_state.renderer.update_uniforms(&app_state.camera);
                        app_state.mouse_state.position.x = position.x as f32;
                        app_state.mouse_state.position.y = position.y as f32;
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app);

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    run()
}
