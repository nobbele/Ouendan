struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct ProjectionUniform {
    matrix: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> proj: ProjectionUniform;

struct ProjectionUniform {
    matrix: mat4x4<f32>;
    source: vec4<f32>;
};
[[group(1), binding(0)]]
var<uniform> view: ProjectionUniform;

struct InstanceInput {
    [[location(2)]] model_matrix_0: vec4<f32>;
    [[location(3)]] model_matrix_1: vec4<f32>;
    [[location(4)]] model_matrix_2: vec4<f32>;
    [[location(5)]] model_matrix_3: vec4<f32>;
    [[location(6)]] source: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.uv = model.uv * instance.source.zw * view.source.zw + instance.source.xy + view.source.xy;
    out.clip_position = proj.matrix * view.matrix * model_matrix * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

[[group(2), binding(0)]]
var texture: texture_2d<f32>;
[[group(2), binding(1)]]
var t_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let sample = textureSample(texture, t_sampler, in.uv);
    if (sample.w <= 0.0) {
        discard;
    }
    return sample;
}