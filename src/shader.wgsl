@group(0) @binding(0)
var block_texture: texture_2d<f32>;

@group(0) @binding(1)
var block_sampler: sampler;

struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) colour: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.colour = model.colour;
    out.uv = model.uv;
    out.world_position = model.position;
    out.clip_position = camera.view_projection * vec4<f32>(model.position, 1.0);

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    let texture_colour = textureSample(
        block_texture,
        block_sampler,
        in.uv
    );

    var face_normal = normalize(cross(
        dpdy(in.world_position),
        dpdx(in.world_position),
    ));
    if !front_facing {
        face_normal = -face_normal;
    }

    let sun_direction = normalize(vec3<f32>(-0.4, 0.8, 0.3));
    let ambient_light = 0.35;
    let diffuse_light = max(dot(face_normal, sun_direction), 0.0);
    let light = ambient_light + (1.0 - ambient_light) * diffuse_light;

    return vec4<f32>(
        texture_colour.rgb * in.colour * light,
        texture_colour.a,
    );
}