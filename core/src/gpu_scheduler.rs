use shared::{GpuRequest, Task, TaskResult};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, debug};
use anyhow::Result;

/// GPU compute scheduler using wgpu.
/// Dispatches WGSL compute shaders across Vulkan, Metal, DX12, and WebGPU backends.
pub struct GpuEngine {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    adapter_info: wgpu::AdapterInfo,
    shader_cache: Arc<Mutex<std::collections::HashMap<String, wgpu::ComputePipeline>>>,
}

impl GpuEngine {
    pub async fn new() -> Result<Self> {
        info!("Initializing wgpu GPU scheduler...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await.ok_or_else(|| anyhow::anyhow!("No GPU adapter found"))?;

        let info = adapter.get_info();
        info!("GPU: {} | Backend: {:?} | Driver: {}", info.name, info.backend, info.driver);

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("LoomGpuDevice"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ).await?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info: info,
            shader_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_info.name
    }

    pub fn backend(&self) -> wgpu::Backend {
        self.adapter_info.backend
    }

    pub async fn enqueue(&self, task: Task) -> Result<TaskResult> {
        let req: GpuRequest = serde_json::from_slice(&task.payload)
            .map_err(|e| anyhow::anyhow!("Deserialize GpuRequest failed: {}", e))?;

        info!("GPU task {}: shader={}, buffers={}, output={}",
            task.task_id, req.shader_id, req.input_buffers.len(), req.output_size);

        let result = self.dispatch_compute(&req).await?;

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("gpu_time_ms".into(), result.1);
        metrics.insert("adapter".into(), if self.adapter_info.backend == wgpu::Backend::Vulkan { 1.0 } else { 0.0 });

        Ok(TaskResult {
            task_id: task.task_id,
            success: true,
            output: result.0,
            metrics,
        })
    }

    async fn get_pipeline(&self, shader_id: &str, wgsl_source: &str) -> Result<wgpu::ComputePipeline> {
        let mut cache = self.shader_cache.lock().await;
        if let Some(pipeline) = cache.get(shader_id) {
            debug!("Shader cache hit: {}", shader_id);
            return Ok(pipeline.clone());
        }

        let shader_module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_id),
            source: wgpu::ShaderSource::Wgsl(wgsl_source.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{}_layout", shader_id)),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_pipeline_layout", shader_id)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("{}_pipeline", shader_id)),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        cache.insert(shader_id.to_string(), pipeline.clone());
        info!("Shader compiled: {}", shader_id);
        Ok(pipeline)
    }

    async fn dispatch_compute(&self, req: &GpuRequest) -> Result<(Vec<u8>, f64)> {
        let start = std::time::Instant::now();

        let input_data: Vec<f32> = req.input_buffers.iter().flat_map(|b| b.clone()).collect();
        let input_bytes = bytemuck::cast_slice(&input_data);

        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input"),
            contents: input_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: (req.output_size * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms"),
            contents: bytemuck::bytes_of(&(req.output_size as u32)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let wgsl_source = self.get_shader_source(&req.shader_id);
        let pipeline = self.get_pipeline(&req.shader_id, &wgsl_source).await?;

        let bind_group_layout = pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: input_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: output_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute_encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((req.output_size as f32) / 256.0).ceil() as u32;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: output_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_buffer.size());
        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.await??;

        let data = buffer_slice.get_mapped_range();
        let result_bytes = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        info!("GPU compute: {} bytes in {:.2}ms", result_bytes.len(), elapsed);

        Ok((result_bytes, elapsed))
    }

    fn get_shader_source(&self, shader_id: &str) -> String {
        match shader_id {
            "generic_map" => include_str!("shaders/compute.wgsl").to_string(),
            "particle_update" => include_str!("shaders/particle_update.wgsl").to_string(),
            _ => include_str!("shaders/compute.wgsl").to_string(),
        }
    }
}
