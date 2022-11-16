use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor,
    ComputePipeline, Device, Queue, ShaderStages,
};

use crate::shader::CellInfo;

/// Hold size information about field
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FieldInfo {
    width: u32,
    height: u32,
}

impl FieldInfo {
    fn create_bind_group(
        binding: u32,
        device: &Device,
        buffer: &Buffer,
    ) -> (BindGroupLayout, BindGroup) {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Life's field info bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Life's field info bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }],
        });

        (bind_group_layout, bind_group)
    }
}

/// Hold cell states
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FieldState(Vec<u32>);

impl FieldState {
    fn create_bind_group(
        binding: u32,
        device: &Device,
        buffer: &Buffer,
        read_only: bool,
    ) -> (BindGroupLayout, BindGroup) {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Life's field bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Life's field bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }],
        });

        (bind_group_layout, bind_group)
    }
}

pub struct Life {
    field_info: FieldInfo,
    field_info_buffer: Buffer,
    field_info_bind: BindGroup,

    compute_pipeline: ComputePipeline,

    life_buffer: Buffer,
    life_bind: BindGroup,

    new_life_buffer: Buffer,
    new_life_bind: BindGroup,
}

impl Life {
    pub fn new(width: u32, height: u32, device: &Device) -> Self {
        // Field Info buffer prepare
        let field_info = FieldInfo { width, height };
        let field_info_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("life field info buffer"),
            contents: bytemuck::cast_slice(&[field_info]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let (field_info_bind_layout, field_info_bind) =
            FieldInfo::create_bind_group(0, &device, &field_info_buffer);

        // Current Field State init
        let field = (0..(width * height))
            .map(|_| rand::random::<u32>() % 2)
            .collect::<Vec<_>>();

        let life_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("life buffer"),
            contents: bytemuck::cast_slice(&field),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let (life_bind_layout, life_bind) =
            FieldState::create_bind_group(0, &device, &life_buffer, true);

        // New Field State init
        let new_life_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("new life buffer"),
            contents: bytemuck::cast_slice(&field),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        let (new_life_bind_layout, new_life_bind) =
            FieldState::create_bind_group(0, &device, &new_life_buffer, false);

        // Init Compute pipiline
        let module = device.create_shader_module(include_wgsl!("../shaders/life.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Life Compute pipeline layout"),
            bind_group_layouts: &[
                &field_info_bind_layout, // Group 0
                &life_bind_layout,       // Group 1
                &new_life_bind_layout,   // Group 2
            ],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Life Compute pipeline"),
            layout: Some(&layout),
            module: &module,
            entry_point: "main",
        });

        Self {
            field_info,
            field_info_buffer,
            field_info_bind,

            compute_pipeline,

            life_buffer,
            life_bind,

            new_life_buffer,
            new_life_bind,
        }
    }

    pub fn step(&mut self, queue: &Queue, device: &Device) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.field_info_bind, &[]);
            compute_pass.set_bind_group(1, &self.life_bind, &[]);
            compute_pass.set_bind_group(2, &self.new_life_bind, &[]);
            compute_pass.dispatch_workgroups(self.field_info.width, self.field_info.height, 1)
        }

        // Copy result
        let size =
            self.field_info.width * self.field_info.height * std::mem::size_of::<u32>() as u32;
        encoder.copy_buffer_to_buffer(&self.new_life_buffer, 0, &self.life_buffer, 0, size.into());
        queue.submit(Some(encoder.finish()));

        // device.poll(wgpu::Maintain::Wait);
    }

    #[inline(always)]
    pub fn cell_count(&self) -> usize {
        (self.field_info.width * self.field_info.height) as usize
    }

    #[inline(always)]
    fn index(&self, x: u32, y: u32) -> usize {
        let x_rem = x % self.field_info.width;
        let y_rem = y % self.field_info.height;

        x_rem as usize + y_rem as usize * self.field_info.width as usize
    }

    pub fn generate_cell_info(&self) -> Vec<CellInfo> {
        let mut out = Vec::with_capacity((self.field_info.width * self.field_info.height) as usize);

        for i in 0..self.field_info.width {
            for j in 0..self.field_info.height {
                out.push(CellInfo {
                    pos: [i as f32, j as f32],
                    idx: self.index(i, j) as u32,
                })
            }
        }

        out
    }

    #[inline(always)]
    pub fn life_buffer(&self) -> &Buffer {
        &self.life_buffer
    }
}
