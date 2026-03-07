struct PointLight {
    pos:   vec4<f32>,   // xyz, pad
    color: vec4<f32>,   // rgb, intensity
}

struct MeshUniforms {
    view_proj: mat4x4<f32>,   // 64 bytes
    material:  vec4<f32>,     // RGBA            (bytes 64-79)
    pbr:       vec4<f32>,     // metallic, roughness, texture_id, normal_map_id (bytes 80-95)
    camera_pos: vec4<f32>,    // xyz, pad (bytes 96-111)
    lights:    array<PointLight, 4>, // (32 * 4 = 128 bytes)
}

@group(0) @binding(0)
var<uniform> u: MeshUniforms;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

// Sprint 71: Normal Map Binding
@group(0) @binding(3)
var t_normal: texture_2d<f32>;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    
    // Instance Data (VB1)
    @location(3) i_mat0: vec4<f32>,
    @location(4) i_mat1: vec4<f32>,
    @location(5) i_mat2: vec4<f32>,
    @location(6) i_mat3: vec4<f32>,
    @location(7) i_color: vec4<f32>,
    @location(8) i_pbr: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       world_pos: vec3<f32>,
    @location(1)       normal:    vec3<f32>,
    @location(2)       uv:        vec2<f32>,
    @location(3)       color_off: vec4<f32>,
    @location(4)       pbr:       vec4<f32>,
}

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    
    let instance_mat = mat4x4<f32>(v.i_mat0, v.i_mat1, v.i_mat2, v.i_mat3);
    let world_pos4 = instance_mat * vec4<f32>(v.position, 1.0);
    
    out.clip_pos  = u.view_proj * world_pos4;
    out.world_pos = world_pos4.xyz;
    
    let normal_mat = mat3x3<f32>(v.i_mat0.xyz, v.i_mat1.xyz, v.i_mat2.xyz);
    out.normal    = normalize(normal_mat * v.normal);
    
    out.uv        = v.uv;
    out.color_off = v.i_color;
    out.pbr       = v.i_pbr;
    
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Determine which PBR/Texture settings to use (Instance vs Uniform)
    var pbr = in.pbr;
    if (pbr.z < -0.5) { // Sentinel for "use uniform"
        pbr = u.pbr;
    }

    var albedo = u.material.rgb * in.color_off.rgb;
    let texture_id = pbr.z;
    if (texture_id > 0.5) {
        let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
        albedo = albedo * tex_color.rgb;
    }
    
    let alpha     = u.material.a * in.color_off.a;
    var normal    = normalize(in.normal);

    // Sprint 71: Normal Mapping
    let normal_map_id = pbr.w;
    if (normal_map_id > 0.5) {
        // Simple derivative-based TBN approximation since we lack tangents
        let dp1 = dpdx(in.world_pos);
        let dp2 = dpdy(in.world_pos);
        let duv1 = dpdx(in.uv);
        let duv2 = dpdy(in.uv);

        let n = normalize(in.normal);
        let t = normalize(dp1 * duv2.y - dp2 * duv1.y);
        let b = -normalize(cross(n, t));
        let tbn = mat3x3<f32>(t, b, n);

        let map_normal = textureSample(t_normal, s_diffuse, in.uv).rgb * 2.0 - 1.0;
        normal = normalize(tbn * map_normal);
    }

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
        let roughness = max(pbr.y, 0.01);
        let shininess = 2.0 / (roughness * roughness + 0.001) - 2.0;
        let spec_val = pow(max(dot(normal, half_dir), 0.0), shininess);
        let metallic = pbr.x;
        total_specular += mix(vec3<f32>(0.04), albedo, metallic) * spec_val * attenuation * light.color.a;
    }

    let out_color = ambient + total_diffuse + total_specular;
    return vec4<f32>(clamp(out_color, vec3<f32>(0.0), vec3<f32>(1.0)), alpha);
}
