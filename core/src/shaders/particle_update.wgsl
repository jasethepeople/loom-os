// GPU-accelerated particle system (10,000+ particles)

struct Particle {
    pos: vec4<f32>,
    vel: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> particles_in: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> particles_out: array<Particle>;

@group(0) @binding(2)
var<uniform> time: vec4<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let count = arrayLength(&particles_in);
    if (idx >= count) { return; }

    var p = particles_in[idx];
    let dt = time.x;

    p.vel.y -= 9.8 * dt;
    p.vel.xyz *= 0.995;
    p.pos.xyz += p.vel.xyz * dt;

    if (p.pos.y < 0.0) {
        p.pos.y = 0.0;
        p.vel.y *= -0.8;
    }

    p.vel.w -= dt;
    p.color.a = clamp(p.vel.w / 5.0, 0.0, 1.0);
    particles_out[idx] = p;
}
