// Generic compute shader — element-wise sigmoid + scale
// Binding 0: input buffer (read-only storage)
// Binding 1: output buffer (read-write storage)
// Binding 2: uniform params

@group(0) @binding(0)
var<storage, read> input_buffer: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output_buffer: array<f32>;

@group(0) @binding(2)
var<uniform> params: vec4<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let size = params.x;
    if (idx >= size) { return; }
    let x = input_buffer[idx];
    let sigmoid = 1.0 / (1.0 + exp(-x));
    output_buffer[idx] = sigmoid * 2.0 - 1.0;
}
