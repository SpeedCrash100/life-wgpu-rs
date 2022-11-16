use crate::shader::CellInfo;

pub struct Life {
    field: Vec<Vec<u32>>,
    width: u32,
    height: u32,
}

impl Life {
    pub fn new(width: u32, height: u32) -> Self {
        let field = (0..width)
            .map(|_| {
                (0..height)
                    .map(|_| rand::random::<u32>() % 2)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            field,
            width,
            height,
        }
    }

    pub fn step(&mut self) {
        let mut new_state = self.field.clone();

        for i in 0..self.width {
            for j in 0..self.height {
                let nc = self.neighbor_count(i, j);
                if self.cell(i, j) > 0 && 2 <= nc && nc <= 3 {
                    // Will survive
                } else if self.cell(i, j) == 0 && nc == 3 {
                    new_state[i as usize][j as usize] = 1;
                } else {
                    new_state[i as usize][j as usize] = 0;
                }
            }
        }

        self.field = new_state;
    }

    fn cell(&self, x: u32, y: u32) -> u32 {
        let x_rem = x % self.width;
        let y_rem = y % self.height;

        self.field[x_rem as usize][y_rem as usize]
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
                    living: self.cell(i, j),
                })
            }
        }

        out
    }
}
