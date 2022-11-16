use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue,
};

use crate::shader::CellInfo;

pub struct Life {
    field: Vec<u32>,
    width: u32,
    height: u32,

    life_buffer: Buffer,
}

impl Life {
    pub fn new(width: u32, height: u32, device: &Device) -> Self {
        let field = (0..(width * height))
            .map(|_| rand::random::<u32>() % 2)
            .collect::<Vec<_>>();

        let life_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("life buffer"),
            contents: bytemuck::cast_slice(&field),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        Self {
            field,
            width,
            height,

            life_buffer,
        }
    }

    pub fn step(&mut self, queue: &Queue) {
        let mut new_state = self.field.clone();

        for i in 0..self.width {
            for j in 0..self.height {
                let nc = self.neighbor_count(i, j);
                let alive = self.cell(i, j) > 0;
                let idx = self.index(i, j);
                if alive && 2 <= nc && nc <= 3 {
                    // Will survive
                } else if !alive && nc == 3 {
                    new_state[idx] = 1
                } else if alive {
                    new_state[idx] = 0
                }
            }
        }

        self.field = new_state;
        queue.write_buffer(&self.life_buffer, 0, bytemuck::cast_slice(&self.field));
    }

    #[inline(always)]
    pub fn cell_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    #[inline(always)]
    fn cell(&self, x: u32, y: u32) -> u32 {
        self.field[self.index(x, y)]
    }

    #[inline(always)]
    fn index(&self, x: u32, y: u32) -> usize {
        let x_rem = x % self.width;
        let y_rem = y % self.height;

        x_rem as usize + y_rem as usize * self.width as usize
    }

    fn neighbor_count(&self, x: u32, y: u32) -> u32 {
        let mut out = 0;
        for i in 0..=2 {
            for j in 0..=2 {
                if i == 1 && j == 1 {
                    continue;
                }

                if x == 0 || y == 0 {
                    continue;
                }

                if self.cell(x + i - 1, y + j - 1) > 0 {
                    out += 1;
                }
            }
        }

        out
    }

    pub fn generate_cell_info(&self) -> Vec<CellInfo> {
        let mut out = Vec::with_capacity((self.width * self.height) as usize);

        for i in 0..self.width {
            for j in 0..self.height {
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
