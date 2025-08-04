use crate::renderer::Renderer;
use std::sync::Arc;
use winit::window::Window;

impl Renderer {
    pub async fn from_winit_window(window: Arc<Window>) -> anyhow::Result<Self> {
        let window_size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        log::debug!("Winit surface created.");
        Self::new_with_surface(surface, instance, window_size.width, window_size.height).await
    }
}
