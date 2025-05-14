// src/rendering/shader.wgsl

// Structures for ColorVertex
struct ColorVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct ColorVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Structures for SpriteVertex
struct SpriteVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct SpriteVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Color Vertex Shader
@vertex
fn vs_color_main(model: ColorVertexInput) -> ColorVertexOutput {
    var out: ColorVertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.color = model.color;
    return out;
}

// Sprite Vertex Shader
@vertex
fn vs_sprite_main(model: SpriteVertexInput) -> SpriteVertexOutput {
    var out: SpriteVertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    return out;
}

// Color Fragment Shader
@fragment
fn fs_color_main(in: ColorVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// Texture and sampler for sprite
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// Sprite Fragment Shader
@fragment
fn fs_sprite_main(in: SpriteVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv);
}
