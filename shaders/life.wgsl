
struct FieldSize {
    width: u32,
    height: u32,
}

@group(0) @binding(0)
var<uniform> field_info: FieldSize;

@group(1) @binding(0)
var<storage, read> life_field: array<u32>;

@group(2) @binding(0)
var<storage, read_write> new_life_field: array<u32>;

fn idx(x: u32, y: u32) -> u32 {
    var x_rem = x % field_info.width;
    var y_rem = y % field_info.height;

    return x_rem + y_rem * field_info.width;
}

fn idx_x(idx: u32) -> u32 {
    return idx % field_info.width;
}

fn idx_y(idx: u32) -> u32 {
    return idx / field_info.width;
}

@compute
@workgroup_size(32)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    var x = global_invocation_id.x;
    var y = global_invocation_id.y;
    var current_idx = idx(x, y);

    // Neighbor count
    var nc: u32 = 0u;
    for (var i = 0u; i <= 2u; i++) {
        for (var j = 0u; j <= 2u; j++) {
            if (i == 1u && j == 1u) {
                continue;
            }

            if ((x == 0u && i == 0u) || (y == 0u && j == 0u)) {
                continue;
            }

            var neighbor_idx = idx(x + i - 1u, y + j - 1u);
            if life_field[neighbor_idx] > 0u {
                nc++;
            }
        }
    }

    // Evaluate new state
    var alive = life_field[current_idx] > 0u;
    if alive && 2u <= nc && nc <= 3u {
        // Will survive
    } else if !alive && nc == 3u {
        new_life_field[current_idx] = 1u;
    } else if alive {
        new_life_field[current_idx] = 0u;
    }


}