use super::camera::Camera;
use super::commands::DrawCommand;
use super::mesh::{Mesh, MeshType, Vertex};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: glam::Mat4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model_matrix: glam::Mat4,
    pub color: glam::Vec4,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Instance {
    pub fn set_position(&mut self, position: glam::Vec3) {
        self.model_matrix.w_axis.x = position.x;
        self.model_matrix.w_axis.y = position.y;
        self.model_matrix.w_axis.z = position.z;
    }

    pub fn from_position_rotation(
        position: glam::Vec3,
        rotation: glam::Mat3,
        scale: f32,
        color: glam::Vec4,
    ) -> Instance {
        let model_matrix = glam::Mat4 {
            x_axis: glam::Vec4::new(
                rotation.x_axis.x * scale,
                rotation.x_axis.y,
                rotation.x_axis.z,
                0.0,
            ),
            y_axis: glam::Vec4::new(
                rotation.y_axis.x,
                rotation.y_axis.y * scale,
                rotation.y_axis.z,
                0.0,
            ),
            z_axis: glam::Vec4::new(
                rotation.z_axis.x,
                rotation.z_axis.y,
                rotation.z_axis.z * scale,
                0.0,
            ),
            w_axis: glam::Vec4::new(position.x, position.y, position.z, 1.0),
        };

        Instance {
            model_matrix,
            color,
        }
    }
}

pub struct Renderer {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub solid_pipeline: wgpu::RenderPipeline,
    pub outline_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub commands: Vec<DrawCommand>,
    pub meshes: HashMap<MeshType, Mesh>,
}

impl Renderer {
    pub fn solid_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.01,
                        g: 0.01,
                        b: 0.01,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.solid_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        // Draw meshes
        let mesh_types: Vec<MeshType> = self.meshes.keys().cloned().collect();
        for mesh_type in mesh_types {
            match mesh_type {
                MeshType::Cube => self.render_mesh(&mesh_type, &mut render_pass),
                MeshType::Tetrahedron => self.render_mesh(&mesh_type, &mut render_pass),
                MeshType::Sphere => self.render_mesh(&mesh_type, &mut render_pass),
                _ => log::warn!(
                    "{:?} mesh rendering has not been implemented yet",
                    mesh_type
                ),
            }
        }
    }

    pub fn outline_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.outline_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        // Draw meshes
        let mesh_types: Vec<MeshType> = self.meshes.keys().cloned().collect();
        for mesh_type in mesh_types {
            match mesh_type {
                MeshType::Cube => self.render_outline_mesh(&mesh_type, &mut render_pass),
                MeshType::Tetrahedron => self.render_outline_mesh(&mesh_type, &mut render_pass),
                MeshType::Sphere => self.render_outline_mesh(&mesh_type, &mut render_pass),
                _ => log::warn!(
                    "{:?} mesh rendering has not been implemented yet",
                    mesh_type
                ),
            }
        }
    }

    pub fn update_depth_texture(&mut self, window_size: glam::UVec2) {
        log::debug!("Redarw depth buffer to size: {}", window_size);
        // Depth Buffer
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: window_size.x,
                height: window_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth24Plus],
        });
        self.depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn update_uniforms(&mut self, camera: &impl Camera) {
        let uniforms = Uniforms {
            view_proj: camera.proj_matrix() * camera.view_matrix(),
        };
        log::trace!("Uniforms: {}", uniforms.view_proj);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub async fn from_winit(window: Arc<winit::window::Window>) -> anyhow::Result<Self> {
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
        log::debug!("Surface created.");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        // Depth Buffer
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: window_size.width.max(1),
                height: window_size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth24Plus],
        });
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Buffers
        //let aspect = window_size.width as f32 / window_size.height as f32;
        let aspect = 2.0;
        let proj_matrix = glam::Mat4::perspective_rh(f32::to_radians(60.0), aspect, 0.1, 1000.0);
        let view_matrix = glam::Mat4::IDENTITY;
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                view_proj: proj_matrix * view_matrix,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        log::debug!("Initial view: {:?}", proj_matrix * view_matrix);

        // Meshes
        let meshes: HashMap<MeshType, Mesh> = [
            (MeshType::Cube, Mesh::new_cube(&device)),
            (MeshType::Tetrahedron, Mesh::new_tetrahedron(&device)),
            (MeshType::Sphere, Mesh::new_sphere(&device, 10)),
        ]
        .into_iter()
        .collect();

        // Solid Bind Groups
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Unforms Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Solid Render Pipeline
        let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("vert_shader.wgsl").into()),
        });
        let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("frag_shader.wgsl").into()),
        });

        let solid_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Solid Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let solid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Solid Pipeline"),
            layout: Some(&solid_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Outline Bind Groups
        let outline_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Outline Unforms Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Outline Render Pipeline
        // Note (mmckenna): Reuses solid vertex shader
        let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Outline Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("outline_frag_shader.wgsl").into()),
        });

        // Note (mmckenna): Reuses solid uniform bind group layout
        let outline_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outline Pipeline Layout"),
                bind_group_layouts: &[&outline_uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let outline_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Outline Pipeline"),
            layout: Some(&outline_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), Instance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::Zero,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            adapter,
            device,
            queue,
            surface,
            surface_config,
            depth_texture,
            depth_texture_view,
            solid_pipeline,
            outline_pipeline,
            uniform_buffer,
            uniform_bind_group,
            meshes,
            commands: Vec::new(),
        })
    }

    pub fn render_mesh(&mut self, mesh_type: &MeshType, render_pass: &mut wgpu::RenderPass<'_>) {
        let mesh = match self.meshes.get_mut(mesh_type) {
            Some(mesh) => mesh,
            None => return,
        };

        let instances: Vec<Instance> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                if &cmd.mesh_type == mesh_type {
                    Some(cmd.instance)
                } else {
                    None
                }
            })
            .collect();

        if instances.len() > mesh.instance_capacity {
            mesh.realloc_instance_buffer(&self.device, instances.len());
        }
        // Write instances to the buffer
        self.queue
            .write_buffer(&mesh.instance_buffer, 0, bytemuck::cast_slice(&instances));

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, mesh.instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..instances.len() as u32);
    }

    pub fn render_outline_mesh(
        &mut self,
        mesh_type: &MeshType,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        let mesh = match self.meshes.get_mut(mesh_type) {
            Some(mesh) => mesh,
            None => return,
        };

        let instances: Vec<Instance> = self
            .commands
            .iter_mut()
            .filter_map(|cmd| {
                if &cmd.mesh_type == mesh_type {
                    let mut wire_instance = cmd.instance;
                    wire_instance.color = glam::Vec4::splat(1.0);
                    wire_instance.model_matrix *= glam::Mat4::from_scale(glam::Vec3::splat(1.005));
                    Some(wire_instance)
                } else {
                    None
                }
            })
            .collect();

        if instances.len() > mesh.edge_instance_capacity {
            mesh.realloc_edge_instance_buffer(&self.device, instances.len());
        }
        // Write instances to the buffer
        self.queue.write_buffer(
            &mesh.edge_instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
        );

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, mesh.edge_instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.edge_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..mesh.edge_indices.len() as u32,
            0,
            0..instances.len() as u32,
        );
    }
}

pub struct OffscreenRenderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub solid_pipeline: wgpu::RenderPipeline,
    pub outline_pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub commands: Vec<DrawCommand>,
    pub meshes: HashMap<MeshType, Mesh>,
}
