struct PointLight {
    pos: vec4<f32>,
    color: vec4<f32>,
}

struct MeshUniforms {
    view_proj: mat4x4<f32>,   // 64 bytes
    material:  vec4<f32>,     // RGBA
    pbr:       vec4<f32>,     // metallic, roughness, texture_id, normal_map_id
    camera_pos: vec4<f32>,    // xyz, pad
    lights:    array<PointLight, 4>,
}

@group(0) @binding(0)
var<uniform> u: MeshUniforms;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;
@group(0) @binding(3)
var t_normal: texture_2d<f32>;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       world_pos: vec3<f32>,
    @location(1)       normal:    vec3<f32>,
    @location(2)       uv:        vec2<f32>,
}

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_pos  = u.view_proj * vec4<f32>(v.position, 1.0);
    out.world_pos = v.position;
    out.normal    = normalize(v.normal);
    out.uv        = v.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var albedo = u.material.rgb;
    let alpha  = u.material.a;
    
    // Texture sampling logic
    if (u.pbr.z >= 0.0) {
        albedo = albedo * textureSample(t_diffuse, s_diffuse, in.uv).rgb;
    }

    let normal = normalize(in.normal);
    let view_dir = normalize(u.camera_pos.xyz - in.world_pos);
    
    var total_diffuse = vec3<f32>(0.0);
    var total_specular = vec3<f32>(0.0);
    let ambient = albedo * 0.15;

    for (var i = 0; i < 4; i++) {
        let light = u.lights[i];
        let light_dir = normalize(light.pos.xyz - in.world_pos);
        let dist = distance(light.pos.xyz, in.world_pos);
        let attenuation = 1.0 / (1.0 + 0.1 * dist + 0.02 * dist * dist);
        
        let n_dot_l = max(dot(normal, light_dir), 0.0);
        total_diffuse += light.color.rgb * albedo * n_dot_l * attenuation * light.color.a;

        let half_dir = normalize(light_dir + view_dir);
        let roughness = max(u.pbr.y, 0.01);
        let shininess = 2.0 / (roughness * roughness + 0.001) - 2.0;
        let spec_val = pow(max(dot(normal, half_dir), 0.0), shininess);
        let metallic = u.pbr.x;
        total_specular += mix(vec3<f32>(0.04), albedo, metallic) * spec_val * attenuation * light.color.a;
    }

    let out_color = ambient + total_diffuse + total_specular;
    return vec4<f32>(clamp(out_color, vec3<f32>(0.0), vec3<f32>(1.0)), alpha);
}
