struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(2) instance_pos: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.position + instance.instance_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    
    // Simple directional lighting based on normal
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(model.normal, light_dir), 0.2);
    
    // Base block color
    let base_color = vec3<f32>(0.2, 0.8, 0.3); // Greenish cubes
    out.color = base_color * diffuse;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
