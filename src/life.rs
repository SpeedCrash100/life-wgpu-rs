use std::sync::Arc;

use glam::Vec2;
use wgpu::{
    include_wgsl, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, Device, Queue,
};

use crate::bindable::{
    BindableToComputePass, CellPos, CellPosInstances, FieldInfo, FieldState, HaveBindGroup,
};

pub struct Life {
    field_info: FieldInfo,

    compute_pipeline: ComputePipeline,

    life: Arc<FieldState>,
    new_life: FieldState,
}

impl Life {
    pub fn new(width: u32, height: u32, device: &Device) -> Self {
        // Field Info buffer prepare
        let field_info = FieldInfo::new(width, height, device);
        let field_info_bind_layout = field_info.get_bind_layout();

        // Current Field State init
        let field = (0..(width * height))
            .map(|_| rand::random::<u32>() % 2)
            .collect::<Vec<_>>();

        let life = Arc::new(FieldState::new(&field, device, true));
        let new_life = FieldState::new(&field, device, false);

        // Init Compute pipiline
        let module = device.create_shader_module(include_wgsl!("../shaders/life.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Life Compute pipeline layout"),
            bind_group_layouts: &[
                &field_info_bind_layout,    // Group 0
                life.get_bind_layout(),     // Group 1
                new_life.get_bind_layout(), // Group 2
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

            compute_pipeline,

            life,
            new_life,
        }
    }

    pub fn step(&mut self, queue: &Queue, device: &Device) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut compute_pass =
                encoder.begin_compute_pass(&ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(&self.compute_pipeline);
            self.field_info
                .bind_to_compute_pass(&mut compute_pass, 0, &[]);

            self.life.bind_to_compute_pass(&mut compute_pass, 1, &[]);
            self.new_life
                .bind_to_compute_pass(&mut compute_pass, 2, &[]);

            compute_pass.dispatch_workgroups(self.field_info.width(), self.field_info.height(), 1)
        }

        // Copy result
        self.life.copy_from(&self.new_life, &mut encoder);
        queue.submit(Some(encoder.finish()));
    }

    #[inline(always)]
    pub fn cell_count(&self) -> usize {
        (self.field_info.width() * self.field_info.height()) as usize
    }

    #[inline(always)]
    fn index(&self, x: u32, y: u32) -> usize {
        let x_rem = x % self.field_info.width();
        let y_rem = y % self.field_info.height();

        x_rem as usize + y_rem as usize * self.field_info.width() as usize
    }

    pub fn generate_cell_info(&self, view_box: (Vec2, Vec2), device: &Device) -> CellPosInstances {
        let mut positions = Vec::with_capacity(self.cell_count() as usize);

        let min_x = view_box.0.x.floor().max(0.0) as u32;
        let max_x = view_box
            .1
            .x
            .floor()
            .min((self.field_info.width() - 1) as f32) as u32;

        let min_y = view_box.0.y.floor().max(0.0) as u32;
        let max_y = view_box
            .1
            .y
            .floor()
            .min((self.field_info.height() - 1) as f32) as u32;

        for i in min_x..=max_x {
            for j in min_y..=max_y {
                positions.push(CellPos {
                    pos: [i as f32, j as f32],
                    idx: self.index(i, j) as u32,
                })
            }
        }

        CellPosInstances::new(positions, device)
    }

    #[inline(always)]
    pub fn life_buffer(&self) -> Arc<FieldState> {
        self.life.clone()
    }
}
